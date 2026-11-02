#define _GNU_SOURCE
#include "platform.h"
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
int main(void) {
    char path[PATH_MAX];
    ssize_t n=readlink("/proc/self/exe",path,sizeof(path)-1);
    if(n<0 || n>=(ssize_t)sizeof(path)-1) return 1;
    path[n]=0;
    char *slash=strrchr(path,'/');
    if(!slash) return 1;
    *slash=0;
    if(chdir(path)) return 1;
    umask(0077);
    if(!freopen("/tmp/ps5remote-m0.log","w",stderr)) return 1;
    setvbuf(stderr,NULL,_IOLBF,0);
    fprintf(stderr,"M0 v0.1.0 uid=%ld euid=%ld cwd=%s\n",(long)getuid(),(long)geteuid(),path);
    return m0_run();
}
