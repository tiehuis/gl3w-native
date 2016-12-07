//! gl3w-native
//! ===========
//!
//! This binary is a port of [gl3w](https://github.com/skaslev/gl3w) to Rust.
//!
//! This removes the python dependency in favour of a statically linked
//! executable (which is hopefully fairly small).
//!
//! Rust is chosen simply because it provides easy cross-platform support and
//! has no real runtime.

extern crate hyper;
extern crate regex;

use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::PathBuf;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct Proc(String, String, String);

impl Proc {
    fn new(id: &str) -> Proc {
        Proc(
            id.to_string(),
            "gl3w".to_string() + &id[2..],
            "PFN".to_string() + &id.to_uppercase() + "PROC"
        )
    }
}

#[derive(Debug)]
enum Gl3wPath {
    /// Represents a single header version
    #[allow(dead_code)]
    Single(PathBuf),

    /// Represents a *.h/*.c file pair
    Separate(PathBuf, PathBuf)
}

/// An `ExecEngine` will run the required commands based on the options it
/// was initialized with.
#[derive(Debug)]
struct Gl3wExec {
    /// Where we should get the glcorearb.h file from.
    url_glcorearb: String,

    /// Where the glcorearb file should be located at
    path_glcorearb: PathBuf,

    /// Where the gl3w.* files should be stored at
    path_gl3w: Gl3wPath,

    /// Bypass the cache and get all files remotely
    no_cache: bool
}

impl Default for Gl3wExec {
    fn default() -> Gl3wExec {
        Gl3wExec {
            url_glcorearb: "https://www.opengl.org/registry/api/GL/glcorearb.h".to_string(),
            path_glcorearb: PathBuf::from("include/GL/glcorearb.h"),
            path_gl3w: Gl3wPath::Separate(
                PathBuf::from("src/gl3w.h"),
                PathBuf::from("src/gl3w.c")
            ),
            no_cache: false
        }
    }
}

impl Gl3wExec {
    pub fn get_glcorearb_h(&self) -> io::Result<String> {
        // Create all directories required by the specified options
        //
        // Note: These unwraps are safe as we ensure after parsing options that
        // we have files at the end.
        fs::create_dir_all(self.path_glcorearb.parent().unwrap())?;

        match self.path_gl3w {
            Gl3wPath::Single(ref path) => {
                fs::create_dir_all(path.parent().unwrap())?;
            }

            Gl3wPath::Separate(ref path_h, ref path_c) => {
                fs::create_dir_all(path_h.parent().unwrap())?;
                fs::create_dir_all(path_c.parent().unwrap())?;
            }
        }

        let mut glcorearb_h = String::new();

        if self.no_cache || !self.path_glcorearb.exists() {
            let client = hyper::Client::new();
            let mut resp = client.get(&self.url_glcorearb).send().unwrap();
            let _ = resp.read_to_string(&mut glcorearb_h)?;

            // We need to write out to the required file as well
            let mut f = File::create(&self.path_glcorearb)?;
            let _ = f.write_all(glcorearb_h.as_bytes())?;
        }
        else {
            // Read file into memory
            let mut f = File::open(&self.path_glcorearb)?;
            let _ = f.read_to_string(&mut glcorearb_h)?;
        }

        Ok(glcorearb_h)
    }

    /// This is a associated function now for consistency and potential
    /// future changes.
    pub fn gen_procs(&self, glcorearb_h: &str) -> Vec<Proc> {
        let re = regex::Regex::new(r"GLAPI.*APIENTRY\s+(\w+)").unwrap();
        let mut procs = re.captures_iter(glcorearb_h)
                                    .map(|e| Proc::new(&e[1]))
                                    .collect::<Vec<_>>();

        procs.sort();
        procs
    }

    /// Generate the required files from the specified proc.
    ///
    /// Return Ok if successfull else error.
    pub fn gen(&self, procs: &[Proc]) -> io::Result<()> {
        match self.path_gl3w {
            Gl3wPath::Single(ref path) => {
                let mut f = File::create(path)?;
                gen_gl3w_single(&mut f, &procs)?;
            }

            Gl3wPath::Separate(ref path_h, ref path_c)  => {
                let mut f_h = File::create(path_h)?;
                let mut f_c = File::create(path_c)?;
                gen_gl3w_h(&mut f_h, &procs)?;
                gen_gl3w_c(&mut f_c, &procs)?;
            }
        }

        Ok(())
    }
}

/// Generate gl3w.h from a list of procs.
fn gen_gl3w_h<T: Write>(out: &mut T, procs: &[Proc]) -> io::Result<()>
{
    write!(out, "{}", include_str!("template/gl3w.preamble.c"))?;
    write!(out, "{}", include_str!("template/gl3w.header.h"))?;

    for p in procs {
        writeln!(out, "extern {:<52} {};", p.2, p.1)?;
    }

    writeln!(out, "")?;

    for p in procs {
        writeln!(out, "#define {:<45} {}", p.0, p.2)?;
    }

    writeln!(out, "")?;
    write!(out, "{}", include_str!("template/gl3w.footer.h"))?;
    Ok(())
}

/// Generate gl3w.c from a list of procs.
fn gen_gl3w_c<T: Write>(out: &mut T, procs: &[Proc]) -> io::Result<()>
{
    write!(out, "{}", include_str!("template/gl3w.preamble.c"))?;
    write!(out, "{}", include_str!("template/gl3w.header.c"))?;

    for p in procs {
        writeln!(out, "{:<52} {};", p.2, p.1)?;
    }

    writeln!(out, "")?;
    writeln!(out, "static void load_procs(void)\n{{")?;

    for p in procs {
        writeln!(out, r#"    {} = ({}) get_proc("{}");"#, p.1, p.2, p.0)?;
    }

    writeln!(out, "}}")?;
    Ok(())
}

/// Generate a combined gl3w.h from a list of procs.
///
/// This is based on gl3w-Single-File.
fn gen_gl3w_single<T: Write>(out: &mut T, procs: &[Proc]) -> io::Result<()>
{
    gen_gl3w_h(out, procs)?;

    writeln!(out, r#"
#if defined(GL3W_IMPLEMENTATION) && !defined(GL3W_IMPLEMENTATION_DONE)
#define GL3W_IMPLEMENTATION_DONE
"#
    )?;

    gen_gl3w_c(out, procs)?;

    writeln!(out, r#"
#endif /* GL3W_IMPLEMENTATION */
"#
    )?;

    Ok(())
}

fn main() {
    let exec = Gl3wExec::default();

    let glcorearb_h = match exec.get_glcorearb_h() {
        Ok(s) => s,
        Err(e) => {
            println!("error: {}", e);
            return;
        }
    };

    // Should always succeed
    let procs = exec.gen_procs(&glcorearb_h);

    match exec.gen(&procs) {
        Ok(_) => (),
        Err(e) => {
            println!("error: {}", e);
            return;
        }
    }
}
