#!/usr/bin/env python3
"""Build a portable review/source archive with pinned source snapshots and Cargo vendor tree."""
import pathlib,shutil,json,hashlib,subprocess,io,tarfile,os
from source_snapshot import NAME,verify
root=pathlib.Path(__file__).resolve().parents[1]
out=root/'build/release-source/ps5remote-webos'
if out.exists():shutil.rmtree(out)
out.mkdir(parents=True)
exclude={'build','target','dist','artifacts','upstream','.tools','.git','__pycache__'}
for current,dirs,files in os.walk(root):
    dirs[:]=[d for d in dirs if d not in exclude]
    rel=pathlib.Path(current).relative_to(root)
    for f in files:
        dest=out/rel/f;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(pathlib.Path(current)/f,dest)
lock=json.loads((root/'upstream.lock.json').read_text())
def archive_git(src,dest,rev):
    data=subprocess.check_output(['git','-C',str(src),'archive',rev])
    with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as t:
        for m in t.getmembers():
            rel=pathlib.PurePosixPath(m.name)
            assert not rel.is_absolute() and '..' not in rel.parts
            p=dest/str(rel)
            if m.isdir():p.mkdir(parents=True,exist_ok=True)
            elif m.isfile():p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read());p.chmod(m.mode)
            else:raise RuntimeError('Unexpected source entry '+m.name)
for name,spec in lock.items():
    if spec['role']=='reference only':continue
    dest=out/'upstream'/name;dest.mkdir(parents=True)
    if name=='ps5upload':
        src=root/'upstream/ps5upload/engine/crates/ps5upload-core/src'
        for p in (root/'crates/psremote/src').rglob('*'):
            if p.is_file() and p.name!='lib.rs':
                rel=p.relative_to(root/'crates/psremote/src');target=dest/'engine/crates/ps5upload-core/src'/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src/rel,target)
        shutil.copytree(root/'upstream/ps5upload/LICENSES',dest/'LICENSES')
    else:archive_git(root/'upstream'/name,dest,spec['rev'])
    for sub,rev in spec.get('submodules',{}).items():archive_git(root/'upstream'/name/sub,dest/sub,rev)
    files={str(p.relative_to(dest)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(dest.rglob('*')) if p.is_file()}
    meta={'url':spec['url'],'revision':spec['rev'],'submodules':spec.get('submodules',{}),'files':files}
    (dest/NAME).write_text(json.dumps(meta,indent=2)+'\n');verify(dest,spec)
shutil.copytree(root/'build/source-vendor',out/'cargo-vendor')
(out/'.cargo').mkdir(exist_ok=True)
(out/'.cargo/config.toml').write_text('[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "cargo-vendor"\n')
(root/'dist').mkdir(exist_ok=True)
archive=root/'dist/ps5remote-webos-0.2.0-source.tar.gz'
with tarfile.open(archive,'w:gz') as t:t.add(out,arcname='ps5remote-webos')
print(archive,archive.stat().st_size,'bytes; source snapshots and Cargo dependency sources included')
