"""Send one command to the isolated Factorio regression scenario over loopback UDP."""
import argparse, json, socket
p=argparse.ArgumentParser()
p.add_argument('command',choices=['panel','stress','encyclopedia','combinator'])
p.add_argument('--port',type=int,default=34210)
a=p.parse_args()
with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as s:
 s.settimeout(10)
 s.sendto(json.dumps({'command':a.command}).encode(),('127.0.0.1',a.port))
 print(s.recv(65535).decode())
