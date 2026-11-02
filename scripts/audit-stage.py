#!/usr/bin/env python3
"""Fail packaging on missing fixture/backend, host binaries, or unintended codec backends."""
import pathlib, subprocess, json, hashlib
root=pathlib.Path(__file__).resolve().parents[1]
stage=root/'build/stage'
for f in ['m0','appinfo.json','icon.png','assets/test.h264','assets/gamecontrollerdb.txt','assets/gamecontrollerdb_extra.txt','lib/ss4s-ndl-webos5.so','lib/libSDL2-2.0.so.0','lib/libSDL2-webos.so.0','lib/libSDL3.so.0','lib/libatomic.so.1','lib/libgcc_s.so.1']:
    assert (stage/f).is_file(), 'missing '+f
assert json.loads((stage/'appinfo.json').read_text())['main']=='m0'
for f in stage.rglob('*'):
    if f.is_file() and f.read_bytes()[:4]==b'\x7fELF':
        b=f.read_bytes()
        assert b[4]==1 and int.from_bytes(b[18:20],'little')==40,'not ELF32 ARM: '+str(f)
        print(subprocess.check_output(['readelf','-d',str(f)],text=True))
assert not list((stage/'lib').glob('ss4s-dummy*'))
assert b'SDL_webOSCreateExportedWindow' in (stage/'lib/libSDL3.so.0').read_bytes(), 'missing SDL3 webOS export API'
assert b'libSDL2-webos.so.0' in (stage/'m0').read_bytes(), 'app must use compatibility library'
assert b'libSDL3.so.0' in (stage/'lib/libSDL2-webos.so.0').read_bytes(), 'compatibility SDL3 loader missing'
# The private NDL shim is intentionally small; never ship SDK dummy SDL under this SONAME.
assert (stage/'lib/libSDL2-2.0.so.0').stat().st_size < 100000, 'expected Aurora NDL shim'
allowed_tv={'libc.so.6','libpthread.so.0','libdl.so.2','librt.so.1','libm.so.6','ld-linux.so.3','libNDL_directmedia.so.1'}
import re
for f in stage.rglob('*'):
    if f.is_file() and f.read_bytes()[:4]==b'\x7fELF':
        deps=re.findall(r'NEEDED.*?\[(.*?)\]',subprocess.check_output(['readelf','-d',str(f)],text=True))
        for dep in deps:
            assert dep in allowed_tv or (stage/'lib'/dep).is_file(), 'unresolved package dependency '+dep
        assert int.from_bytes(f.read_bytes()[36:40],'little') & 0x400 == 0, 'unexpected hard-float ABI'
print('Dependency closure checked against explicit TV library allowlist (TV availability still pending)')
for f in ['m0','lib/ss4s-ndl-webos5.so','lib/libSDL2-2.0.so.0','assets/test.h264']:
    print(hashlib.sha256((stage/f).read_bytes()).hexdigest(),f)
print('Stage ELF/asset audit passed; TV runtime libraries and physical output UNVERIFIED')
