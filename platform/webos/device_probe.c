#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <glob.h>
#include <linux/input.h>
#include <linux/hidraw.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>
/* No chmod, privilege changes, HID output writes, or automatic grabs. */
void probe_devices(void) {
    const char *patterns[]={"/dev/input/event*","/dev/hidraw*"};
    for(unsigned p=0;p<2;p++) {
        glob_t g={0}; int rc=glob(patterns[p],0,NULL,&g);
        fprintf(stderr,"M0 probe pattern=%s glob_rc=%d count=%zu\n",patterns[p],rc,g.gl_pathc);
        for(size_t i=0;i<g.gl_pathc;i++) {
            const char *path=g.gl_pathv[i]; errno=0;
            int fd=open(path,O_RDONLY|O_NONBLOCK|O_CLOEXEC), re=fd<0?errno:0;
            if(fd>=0) {
                if(p==0) { struct input_id id={0}; int r=ioctl(fd,EVIOCGID,&id); fprintf(stderr,"M0 bus path=%s rc=%d bus=%04x vendor=%04x product=%04x\n",path,r,id.bustype,id.vendor,id.product); }
                else {struct hidraw_devinfo id={0}; int r=ioctl(fd,HIDIOCGRAWINFO,&id);fprintf(stderr,"M0 hid path=%s rc=%d bus=%x vendor=%04hx product=%04hx\n",path,r,id.bustype,id.vendor,id.product);}
                close(fd);
            }
            errno=0; fd=open(path,O_RDWR|O_NONBLOCK|O_CLOEXEC); int we=fd<0?errno:0;
            if(fd>=0) close(fd);
            fprintf(stderr,"M0 access path=%s read_errno=%d rw_errno=%d\n",path,re,we);
        }
        globfree(&g);
    }
}
/* Momentary diagnostic only. Never held during normal input. Explicit node required. */
int probe_grab(const char *path) {
    if(!path || strncmp(path,"/dev/input/event",16)) {fprintf(stderr,"M0 grab disabled (M0_GRAB_DEVICE unset/invalid)\n");return -1;}
    int fd=open(path,O_RDONLY|O_NONBLOCK|O_CLOEXEC);
    if(fd<0){fprintf(stderr,"M0 grab open path=%s errno=%d\n",path,errno);return -1;}
    int rc=ioctl(fd,EVIOCGRAB,1), err=rc<0?errno:0;
    fprintf(stderr,"M0 grab path=%s acquire_rc=%d errno=%d\n",path,rc,err);
    if(rc==0){int release=ioctl(fd,EVIOCGRAB,0);fprintf(stderr,"M0 grab release_rc=%d errno=%d\n",release,release<0?errno:0);}
    close(fd); return rc;
}
