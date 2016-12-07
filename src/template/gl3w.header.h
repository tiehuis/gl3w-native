#ifndef __gl3w_h_
#define __gl3w_h_

#include <GL/glcorearb.h>

#ifndef __gl_h_
#define __gl_h_
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*GL3WglProc)(void);

/* gl3w api */
int gl3wInit(void);
int gl3wIsSupported(int major, int minor);
GL3WglProc gl3wGetProcAddress(const char *proc);

/* OpenGL functions */
