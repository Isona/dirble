#!/usr/bin/env python3

import asyncio
from aiohttp import web
import subprocess

async def chunk_handler(request):
    res = web.StreamResponse(
        status=200,
        reason="OK",
        headers={"Content-Type": "text/plain"}
    )
    await res.prepare(request)
    for i in range(10):
        await res.write(b"This is a line: %d\n"%(i))

    return res

async def build_server(loop, address, port):
    app = web.Application()
    app.router.add_route('GET', "/chunked.html", chunk_handler)
    runner = web.AppRunner(app)
    await runner.setup()

    site = web.TCPSite(runner, address, port)
    await site.start()

if __name__ == '__main__':
    listen_addr = "::1"
    port = 5001
    loop = asyncio.get_event_loop()
    loop.run_until_complete(build_server(loop, listen_addr, port))
    print("Server listening on %s, port %d"%(listen_addr, port))

    try:
        loop.run_forever()
    except KeyboardInterrupt:
        print("Shutting Down!")
        loop.close()