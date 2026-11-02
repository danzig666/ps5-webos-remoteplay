/* Test the reused Aurora writer with an in-memory write sink, never a device. */
#define _GNU_SOURCE
#include <assert.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
static uint8_t last_report[128];
static size_t last_size;
static ssize_t capture_write(int fd,const void *data,size_t size){(void)fd;assert(size<=sizeof(last_report));memcpy(last_report,data,size);last_size=size;return (ssize_t)size;}
#define write capture_write
#include "../platform/client/vendor/dualsense_usb.c"
#undef write
int main(void){
    dualsense_usb_t ds={0};uint8_t left[10]={1,2,3},right[10]={4,5,6};
    assert(dualsense_usb_set_adaptive_triggers(&ds,DS_EFFECT_LEFT_TRIGGER|DS_EFFECT_RIGHT_TRIGGER,0x21,0x22,left,right));
    assert(last_size==63 && last_report[0]==0x02);
    assert(last_report[1]==0x0c);
    assert(last_report[11]==0x22 && last_report[12]==4);
    assert(last_report[22]==0x21 && last_report[23]==1);
    assert(dualsense_usb_set_strength(&ds,0x60));assert((last_report[2]&0x40)!=0 && last_report[37]==0x60);
    assert(dualsense_usb_set_lightbar(&ds,10,20,30));assert((last_report[2]&0x04)!=0 && last_report[45]==10 && last_report[47]==30);
    uint8_t off[10]={0};assert(dualsense_usb_set_adaptive_triggers(&ds,0x0c,0,0,off,off));assert(last_report[11]==0 && last_report[22]==0);
    return 0;
}
