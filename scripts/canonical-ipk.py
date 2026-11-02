#!/usr/bin/env python3
"""Normalize only ar/tar/gzip metadata from ares-package, retaining payload and modes."""
import gzip, io, pathlib, tarfile, sys
EPOCH=1793577600

def canonical(data):
    assert data[:8]==b'!<arch>\n'
    pos=8; members=[]
    while pos<len(data):
        h=data[pos:pos+60]; assert h[58:]==b'`\n'
        name=h[:16].decode().strip().rstrip('/')
        size=int(h[48:58]);pos+=60
        content=data[pos:pos+size];pos+=size+(size%2)
        if name.endswith('.tar.gz'):
            out=io.BytesIO()
            with tarfile.open(fileobj=io.BytesIO(content),mode='r:gz') as src,tarfile.open(fileobj=out,mode='w',format=tarfile.GNU_FORMAT) as dst:
                for m in sorted(src.getmembers(),key=lambda x:x.name):
                    m.mtime=EPOCH;m.uid=m.gid=0;m.uname=m.gname='root';m.pax_headers={}
                    dst.addfile(m,src.extractfile(m) if m.isfile() else None)
            content=gzip.compress(out.getvalue(),compresslevel=9,mtime=0)
        members.append((name,content))
    out=bytearray(b'!<arch>\n')
    for name,content in members:
        out.extend(f'{name+"/":<16}{0:<12}{0:<6}{0:<6}{"100644":<8}{len(content):<10}`\n'.encode())
        out.extend(content)
        if len(content)%2:out.extend(b'\n')
    return bytes(out)
if __name__=='__main__':
    for arg in sys.argv[1:]:
        p=pathlib.Path(arg);p.write_bytes(canonical(p.read_bytes()))
