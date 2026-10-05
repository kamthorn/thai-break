#!/usr/bin/env python3
"""
Comprehensive performance benchmark for thai-break-service (flash-alpha)
Measures Latency (p50, p90, p99), Throughput (RPS), and Character Processing Rate.
"""

import asyncio
import statistics
import time
import httpx

BASE_URL = "http://127.0.0.1:8089"

SAMPLE_TEXTS = {
    "short": "ฉันรักภาษาไทยและการเขียนโปรแกรม",
    "medium": "การตัดคำภาษาไทยและการจัดแบ่งบรรทัดอย่างมีประสิทธิภาพสูงด้วยภาษา Rust บน Axum และ Tokio ให้ความเร็วระดับไมโครวินาที",
    "long": (
        "พระบาทสมเด็จพระปรมินทรมหาภูมิพลอดุลยเดช มหิตลาธิเบศรรามาธิบดี จักรีนฤบดินทร สยามินทราธิราช บรมนาถบพิตร "
        "ทรงเป็นพระมหากษัตริย์ไทยรัชกาลที่ ๙ แห่งราชวงศ์จักรี เสด็จสู่สวรรคาลัยเมื่อวันพฤหัสบดีที่ ๑๓ ตุลาคม พ.ศ. ๒๕๕๙ "
        "ณ โรงพยาบาลศิริราช พระชนมพรรษา ๘๘ พรรษา ทรงครองราชสมบัติได้ ๗๐ ปี นับเป็นพระมหากษัตริย์ที่ครองราชย์ยาวนานที่สุดในประเทศไทย "
        "และยาวนานที่สุดในบรรดาพระมหากษัตริย์ในประวัติศาสตร์เอเชียตะวันออกเฉียงใต้"
    ),
    "html": (
        "<div class='article-content'><p>สวัสดีครับ <b>ยินดีต้อนรับ</b> สู่บริการ <a href='#'>ThaiBreak</a> API</p>"
        "<ul><li>ตัดคำรวดเร็ว</li><li>ไม่ทำลาย HTML tags</li><li>ประหยัด RAM ระดับขีดสุด</li></ul></div>"
    ),
}

async def benchmark_endpoint(client, endpoint, payload, iterations=1000):
    latencies = []
    # Warmup
    for _ in range(20):
        await client.post(endpoint, json=payload)

    t_start = time.perf_counter()
    for _ in range(iterations):
        t0 = time.perf_counter()
        resp = await client.post(endpoint, json=payload)
        t1 = time.perf_counter()
        if resp.status_code == 200:
            latencies.append((t1 - t0) * 1000.0) # ms
        else:
            print(f"Error {resp.status_code}: {resp.text}")

    total_time = time.perf_counter() - t_start
    latencies.sort()
    p50 = statistics.median(latencies)
    p90 = latencies[int(len(latencies) * 0.90)]
    p95 = latencies[int(len(latencies) * 0.95)]
    p99 = latencies[int(len(latencies) * 0.99)]
    avg = statistics.mean(latencies)
    rps = len(latencies) / total_time

    return {
        "count": len(latencies),
        "total_time": total_time,
        "rps": rps,
        "avg_ms": avg,
        "p50_ms": p50,
        "p90_ms": p90,
        "p95_ms": p95,
        "p99_ms": p99,
        "min_ms": latencies[0],
        "max_ms": latencies[-1],
    }

async def benchmark_concurrent(endpoint, payload, concurrency=50, total_requests=5000):
    latencies = []
    sem = asyncio.Semaphore(concurrency)

    limits = httpx.Limits(max_keepalive_connections=concurrency * 2, max_connections=concurrency * 2)
    async with httpx.AsyncClient(base_url=BASE_URL, limits=limits, timeout=10.0) as client:
        # Warmup
        for _ in range(20):
            await client.post(endpoint, json=payload)

        async def worker():
            async with sem:
                t0 = time.perf_counter()
                resp = await client.post(endpoint, json=payload)
                t1 = time.perf_counter()
                if resp.status_code == 200:
                    latencies.append((t1 - t0) * 1000.0)

        t_start = time.perf_counter()
        tasks = [asyncio.create_task(worker()) for _ in range(total_requests)]
        await asyncio.gather(*tasks)
        total_time = time.perf_counter() - t_start

    latencies.sort()
    rps = len(latencies) / total_time
    p50 = statistics.median(latencies)
    p90 = latencies[int(len(latencies) * 0.90)]
    p99 = latencies[int(len(latencies) * 0.99)]
    avg = statistics.mean(latencies)

    return {
        "concurrency": concurrency,
        "total_requests": len(latencies),
        "total_time": total_time,
        "rps": rps,
        "avg_ms": avg,
        "p50_ms": p50,
        "p90_ms": p90,
        "p99_ms": p99,
    }

async def main():
    print("=" * 70)
    print("ThaiBreak Service Performance Benchmark (flash-alpha)")
    print(f"Target: {BASE_URL}")
    print("=" * 70)

    limits = httpx.Limits(max_keepalive_connections=100, max_connections=100)
    async with httpx.AsyncClient(base_url=BASE_URL, limits=limits, timeout=10.0) as client:
        # 1. Sequential Latency per Endpoint
        print("\n--- 1. Single-Thread Latency Benchmark (1,000 reqs each) ---")
        endpoints = [
            ("POST /api/v1/words (Short Text)", "/api/v1/words", {"text": SAMPLE_TEXTS["short"]}),
            ("POST /api/v1/words (Medium Text)", "/api/v1/words", {"text": SAMPLE_TEXTS["medium"]}),
            ("POST /api/v1/words (Long Text)", "/api/v1/words", {"text": SAMPLE_TEXTS["long"]}),
            ("POST /api/v1/lines (HTML Text)", "/api/v1/lines", {"text": SAMPLE_TEXTS["html"], "marker": "|", "is_html": True}),
            ("POST /api/v1/wrap (Long Text)", "/api/v1/wrap", {"text": SAMPLE_TEXTS["long"], "width": 60}),
            ("POST /api/v1/compare (Dual-Mode)", "/api/v1/compare", {"text": "เดินทางไปกรมการกงสุลและจุฬาลงกรณ์มหาวิทยาลัย"}),
        ]

        for name, ep, payload in endpoints:
            res = await benchmark_endpoint(client, ep, payload, iterations=1000)
            print(f"\n{name}:")
            print(f"  RPS:    {res['rps']:,.1f} req/s")
            print(f"  Avg:    {res['avg_ms']:.3f} ms | p50: {res['p50_ms']:.3f} ms")
            print(f"  p90:    {res['p90_ms']:.3f} ms | p95: {res['p95_ms']:.3f} ms | p99: {res['p99_ms']:.3f} ms")
            print(f"  Min:    {res['min_ms']:.3f} ms | Max: {res['max_ms']:.3f} ms")

        # 2. Concurrency Scaling Benchmark
        print("\n--- 2. High-Concurrency Throughput Scaling (POST /api/v1/words) ---")
        payload = {"text": SAMPLE_TEXTS["medium"]}
        for c in [10, 25, 50, 100]:
            total = 5000 if c <= 50 else 10000
            res = await benchmark_concurrent("/api/v1/words", payload, concurrency=c, total_requests=total)
            print(f"\nConcurrency {c:3d} (Total {total:,} reqs):")
            print(f"  Throughput: {res['rps']:,.1f} req/sec")
            print(f"  Avg Latency: {res['avg_ms']:.2f} ms | p50: {res['p50_ms']:.2f} ms | p90: {res['p90_ms']:.2f} ms | p99: {res['p99_ms']:.2f} ms")

        # 3. Batch API Benchmark
        print("\n--- 3. Batch API Throughput (texts: [ ... ]) ---")
        batch_sizes = [10, 50, 100]
        for b_size in batch_sizes:
            batch_payload = {"texts": [SAMPLE_TEXTS["short"]] * b_size}
            res = await benchmark_endpoint(client, "/api/v1/words", batch_payload, iterations=200)
            items_per_sec = res['rps'] * b_size
            print(f"\nBatch Size = {b_size} sentences per HTTP request:")
            print(f"  HTTP RPS:         {res['rps']:,.1f} req/s")
            print(f"  Item Throughput:  {items_per_sec:,.1f} sentences/sec")
            print(f"  Avg Latency:      {res['avg_ms']:.2f} ms (p90: {res['p90_ms']:.2f} ms)")

if __name__ == "__main__":
    asyncio.run(main())
