/* Discovery/scan/deduplication adapted from Aurora-TV input_gamepad.c. */
#define _GNU_SOURCE
#include <SDL.h>
#include <ctype.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include "aurora_filter.inc"
#define PADS 4
static SDL_GameController *pads[PADS];
static Uint32 last_scan, last_log;
static int rumble_rc=-999;
static int changed;
void input_scan(void) {
    for(int i=0;i<PADS;i++) if(pads[i] && !SDL_GameControllerGetAttached(pads[i])) {
        fprintf(stderr,"M0 disconnect slot=%d state=released\n",i);
        SDL_GameControllerRumble(pads[i],0,0,0); SDL_GameControllerClose(pads[i]); pads[i]=NULL; changed=1;
    }
    for(int d=0;d<SDL_NumJoysticks();d++) {
        SDL_JoystickID id=SDL_JoystickGetDeviceInstanceID(d);
        int slot=-1,found=0;
        for(int i=0;i<PADS;i++) {if(!pads[i]){if(slot<0)slot=i;} else if(SDL_JoystickInstanceID(SDL_GameControllerGetJoystick(pads[i]))==id)found=1;}
        if(found || slot<0) continue;
        const char *name=SDL_JoystickNameForIndex(d);
        if(webos_name_is_non_gamepad(name)) continue;
        if(!SDL_IsGameController(d)) {fprintf(stderr,"M0 unmapped joystick index=%d id=%d name=%s; mapping needed\n",d,id,name?name:"?");continue;}
        SDL_GameController *pad=SDL_GameControllerOpen(d);
        if(!pad){fprintf(stderr,"M0 controller open failed: %s\n",SDL_GetError());continue;}
        SDL_Joystick *j=SDL_GameControllerGetJoystick(pad);
        if(SDL_JoystickNumAxes(j)<4 || SDL_JoystickNumButtons(j)<8){SDL_GameControllerClose(pad);continue;}
        pads[slot]=pad; changed=1;
        char guid[33]; SDL_JoystickGetGUIDString(SDL_JoystickGetGUID(j),guid,sizeof(guid));
        const char *path=SDL_JoystickPath(j);
        fprintf(stderr,"M0 controller slot=%d id=%d name=%s guid=%s path=%s vendor=%04x product=%04x rumble_advertised=%d transport=verify_bus_log\n",slot,id,name?name:"?",guid,path?path:"unknown",SDL_GameControllerGetVendor(pad),SDL_GameControllerGetProduct(pad),SDL_GameControllerHasRumble(pad));
    }
    last_scan=SDL_GetTicks();
}
void input_rumble(void) {
    for(int i=0;i<PADS;i++)if(pads[i]) {
        rumble_rc=SDL_GameControllerRumble(pads[i],0x6000,0xc000,500);
        fprintf(stderr,"M0 rumble slot=%d rc=%d error=%s physical_result=UNVERIFIED\n",i,rumble_rc,rumble_rc?SDL_GetError():"none");
    }
}
void input_update(void) {
    Uint32 now=SDL_GetTicks();
    if(now-last_scan>2000)input_scan();
    if(changed || now-last_log>1000) {
        for(int i=0;i<PADS;i++)if(pads[i]) {
            unsigned bits=0; for(int b=0;b<SDL_CONTROLLER_BUTTON_MAX;b++)if(SDL_GameControllerGetButton(pads[i],b))bits|=1u<<b;
            fprintf(stderr,"M0 state slot=%d buttons=%08x axes=",i,bits);
            for(int a=0;a<SDL_CONTROLLER_AXIS_MAX;a++)fprintf(stderr,"%d,",SDL_GameControllerGetAxis(pads[i],a));
            fprintf(stderr,"\n");
        }
        last_log=now;changed=0;
    }
}
void input_event(const SDL_Event *e) {
    if(e->type==SDL_CONTROLLERBUTTONDOWN || e->type==SDL_CONTROLLERBUTTONUP) {
        fprintf(stderr,"M0 button id=%d button=%d pressed=%d\n",e->cbutton.which,e->cbutton.button,e->cbutton.state);changed=1;
        if(e->type==SDL_CONTROLLERBUTTONDOWN && e->cbutton.button==SDL_CONTROLLER_BUTTON_START)input_rumble();
    }
    if(e->type==SDL_CONTROLLERAXISMOTION)changed=1;
    if(e->type==SDL_JOYDEVICEADDED || e->type==SDL_JOYDEVICEREMOVED || e->type==SDL_CONTROLLERDEVICEADDED || e->type==SDL_CONTROLLERDEVICEREMOVED)input_scan();
}
void input_draw(SDL_Renderer *r, void (*text)(int,int,const char *)) {
    char line[256];
    text(12,12,"M0 H.264 1080p60 / TRANSPARENT OVERLAY / 120 SECOND RUN");
    text(12,28,"START = 500ms RUMBLE; R/RED = RUMBLE; G/GREEN = OPTIONAL GRAB; ESC/BACK = EXIT");
    snprintf(line,sizeof(line),"RUMBLE API RESULT %d (physical confirmation required)",rumble_rc);text(12,44,line);
    for(int i=0;i<PADS;i++) {
        int y=70+i*100;
        if(!pads[i]){snprintf(line,sizeof(line),"PAD %d: disconnected",i);text(12,y,line);continue;}
        snprintf(line,sizeof(line),"PAD %d: %.75s",i,SDL_GameControllerName(pads[i]));text(12,y,line);
        unsigned bits=0;for(int b=0;b<SDL_CONTROLLER_BUTTON_MAX;b++)if(SDL_GameControllerGetButton(pads[i],b))bits|=1u<<b;
        snprintf(line,sizeof(line),"buttons %08x rumble advertised %d",bits,SDL_GameControllerHasRumble(pads[i]));text(12,y+16,line);
        for(int a=0;a<SDL_CONTROLLER_AXIS_MAX;a++) {
            int v=SDL_GameControllerGetAxis(pads[i],a);
            snprintf(line,sizeof(line),"%d:%6d",a,v);text(12+a*130,y+32,line);
            SDL_SetRenderDrawColor(r,30,220,100,255);SDL_Rect bar={12+a*130,y+50,(v+32768)*110/65535,8};SDL_RenderFillRect(r,&bar);
        }
    }
}
void input_close(void){for(int i=0;i<PADS;i++)if(pads[i]){SDL_GameControllerRumble(pads[i],0,0,0);SDL_GameControllerClose(pads[i]);pads[i]=NULL;}}
