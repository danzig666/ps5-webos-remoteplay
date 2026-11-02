#define _GNU_SOURCE
#include "platform.h"
#include <SDL.h>
#include <SDL_test_font.h>
#include <SDL_webOS.h>
#include <ss4s.h>
#include <signal.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static SDL_Window *window;
static SDL_Renderer *renderer;
static SS4S_Player *player;
static int ss_initialized,video_open,feed_rc=-999;
static volatile sig_atomic_t stopped;
extern void input_scan(void),input_update(void),input_rumble(void),input_close(void),probe_devices(void);
extern int probe_grab(const char *);
extern void input_event(const SDL_Event *);
extern void input_draw(SDL_Renderer *,void (*)(int,int,const char *));
static void stop(int sig){(void)sig;stopped=1;}
static void text(int x,int y,const char *s){SDL_SetRenderDrawColor(renderer,255,255,255,255);SDLTest_DrawString(renderer,x,y,s);}
static void ss_log(SS4S_LogLevel level,const char *tag,const char *fmt,...) {
    fprintf(stderr,"M0 SS4S level=%d tag=%s ",level,tag);
    va_list args;va_start(args,fmt);vfprintf(stderr,fmt,args);va_end(args);fputc('\n',stderr);
}
int platform_open(void) {
    signal(SIGINT,stop);signal(SIGTERM,stop);
    setenv("APPID","org.homebrew.ps5remote.m0",1);
    setenv("SS4S_CONFIG_FILE","./lib/ss4s_modules.ini",1);
    SDL_SetHint("SDL_WEBOS_REGISTER_APP","1");
    if(SDL_Init(SDL_INIT_VIDEO|SDL_INIT_GAMECONTROLLER|SDL_INIT_HAPTIC)<0){fprintf(stderr,"M0 SDL init: %s\n",SDL_GetError());return -1;}
    SDL_version v;SDL_GetVersion(&v);fprintf(stderr,"M0 SDL runtime=%d.%d.%d video=%s\n",v.major,v.minor,v.patch,SDL_GetCurrentVideoDriver());
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE,8);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE,8);SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE,8);SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE,8);
    window=SDL_CreateWindow("M0 hardware feasibility",SDL_WINDOWPOS_UNDEFINED,SDL_WINDOWPOS_UNDEFINED,1920,1080,SDL_WINDOW_OPENGL|SDL_WINDOW_FULLSCREEN_DESKTOP|SDL_WINDOW_SHOWN);
    if(!window){fprintf(stderr,"M0 window: %s\n",SDL_GetError());return -1;}
    renderer=SDL_CreateRenderer(window,-1,SDL_RENDERER_ACCELERATED);
    if(!renderer){fprintf(stderr,"M0 accelerated renderer: %s\n",SDL_GetError());return -1;}
    SDL_RenderSetLogicalSize(renderer,1280,720);
    SDL_RendererInfo ri;SDL_GetRendererInfo(renderer,&ri);
    int alpha=0;SDL_GL_GetAttribute(SDL_GL_ALPHA_SIZE,&alpha);fprintf(stderr,"M0 renderer=%s alpha_bits=%d\n",ri.name,alpha);
    if(alpha<8){fprintf(stderr,"M0 no alpha surface\n");return -1;}
    SDL_SetRenderDrawColor(renderer,0,0,0,0);SDL_RenderClear(renderer);SDL_RenderPresent(renderer);
    fprintf(stderr,"M0 mappings base=%d extra=%d\n",SDL_GameControllerAddMappingsFromFile("assets/gamecontrollerdb.txt"),SDL_GameControllerAddMappingsFromFile("assets/gamecontrollerdb_extra.txt"));
    probe_devices();input_scan();
    // Permit the foreground window to settle before first NDL load (upstream requirement).
    Uint32 until=SDL_GetTicks()+500;while(!SDL_TICKS_PASSED(SDL_GetTicks(),until)){if(platform_tick())return -1;SDL_Delay(5);}
    SS4S_Config config={.audioDriver="ndl-webos5",.videoDriver="ndl-webos5",.loggingFunction=ss_log};
    int rc=SS4S_Init(0,NULL,&config);fprintf(stderr,"M0 SS4S init rc=%d\n",rc);
    if(rc!=0)return -1;
    ss_initialized=1;
    rc=SS4S_PostInit(0,NULL);fprintf(stderr,"M0 SS4S postinit rc=%d\n",rc);if(rc!=0)return -1;
    player=SS4S_PlayerOpen();if(!player){fprintf(stderr,"M0 SS4S player failed\n");return -1;}
    SS4S_PlayerSetWaitAudioVideoReady(player,false); // video-only M0
    SS4S_PlayerSetViewportSize(player,1920,1080);
    SS4S_VideoInfo info={.codec=SS4S_VIDEO_H264,.width=1920,.height=1080,.frameRateNumerator=60,.frameRateDenominator=1};
    rc=SS4S_PlayerVideoOpen(player,&info);fprintf(stderr,"M0 H264 open rc=%d module=%s visual_result=UNVERIFIED\n",rc,SS4S_GetVideoModuleName());
    if(rc!=0)return -1;
    video_open=1;return 0;
}
int platform_tick(void) {
    SDL_Event e;
    while(SDL_PollEvent(&e)) {
        if(e.type==SDL_QUIT)stopped=1;
        if(e.type==SDL_KEYDOWN){
            if(e.key.keysym.sym==SDLK_ESCAPE || e.key.keysym.scancode==SDL_WEBOS_SCANCODE_BACK || e.key.keysym.scancode==SDL_WEBOS_SCANCODE_EXIT)stopped=1;
            if(!e.key.repeat && (e.key.keysym.sym==SDLK_r || e.key.keysym.scancode==SDL_WEBOS_SCANCODE_RED))input_rumble();
            if(!e.key.repeat && (e.key.keysym.sym==SDLK_g || e.key.keysym.scancode==SDL_WEBOS_SCANCODE_GREEN))probe_grab(getenv("M0_GRAB_DEVICE"));
        }
        input_event(&e);
    }
    input_update();
    // Clear entire EGL surface transparent; only diagnostic panel is opaque.
    SDL_SetRenderDrawColor(renderer,0,0,0,0);SDL_RenderClear(renderer);
    SDL_SetRenderDrawColor(renderer,15,25,35,255);SDL_Rect panel={0,0,960,480};SDL_RenderFillRect(renderer,&panel);
    input_draw(renderer,text);
    char line[128];snprintf(line,sizeof(line),"SS4S feed rc=%d; decoded/presented UNKNOWN; verify moving picture below panel",feed_rc);text(12,460,line);
    SDL_RenderPresent(renderer);
    return stopped!=0;
}
int platform_feed(const unsigned char *data,size_t size,int key) {
    SS4S_VideoFeedFlags flags=SS4S_VIDEO_FEED_DATA_FRAME_START|SS4S_VIDEO_FEED_DATA_FRAME_END;
    if(key)flags|=SS4S_VIDEO_FEED_DATA_KEYFRAME;
    return feed_rc=SS4S_PlayerVideoFeed(player,data,size,flags);
}
void platform_close(void) {
    input_close();
    if(video_open){SS4S_PlayerVideoClose(player);video_open=0;}
    if(player){SS4S_PlayerClose(player);player=NULL;}
    if(ss_initialized){SS4S_Quit();ss_initialized=0;}
    if(renderer){SDLTest_CleanupTextDrawing();SDL_DestroyRenderer(renderer);renderer=NULL;}
    if(window){SDL_DestroyWindow(window);window=NULL;}
    SDL_Quit();fprintf(stderr,"M0 shutdown\n");
}
