#!/usr/bin/env python3
"""
灾难恢复脚本：把本地原图按数据库 images.file_path 的文件名推回线上存储。

原理：
- 数据库记录完好（id -> file_path），二维码内容 = {BASE_URL}/view/{id}
- 线上文件丢失，本脚本按 DB 里的 file_path 把原图经 /restore/<key> 接口写回去
- 原图来自本地 picture_* 目录，通过 mapping CSV 的 file_hash (MD5) 校验正确性

用法:
  python restore.py --dry-run     # 只核对覆盖率，不上传
  python restore.py               # 正式恢复
"""

import argparse
import csv
import hashlib
import os
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.parse import urlparse

import requests

BASE_URL = os.environ.get("RESTORE_BASE_URL", "http://localhost:3000")
DELETE_KEY = os.environ.get("RESTORE_DELETE_KEY", "")

SCRIPT_DIR = Path(__file__).resolve().parent
PICTURE_DIRS = [
    SCRIPT_DIR / "picture_0811",
    SCRIPT_DIR / "picture_5_20",
    SCRIPT_DIR / "picture_6_29",
    SCRIPT_DIR / "pictures_5_3",
]
MAPPING_CSVS = [
    SCRIPT_DIR / "QR_0811_mapping.csv",
    SCRIPT_DIR / "QR_5_20_mapping.csv",
    SCRIPT_DIR / "QR_5_3_mapping.csv",
    SCRIPT_DIR / "QR_6_29_mapping.csv",
]
DB_DUMP = Path("/tmp/db_images.csv")  # id,file_path（从线上库 \copy 导出）
ALLOWED_EXT = {".jpg", ".jpeg", ".png", ".gif", ".webp"}


def md5_of(path: Path) -> str:
    h = hashlib.md5()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def build_local_index():
    """扫描本地图片目录: 文件名 -> 路径；返回 (index, 重名文件集合)"""
    index = {}
    dupes = set()
    for d in PICTURE_DIRS:
        if not d.exists():
            print(f"[警告] 目录不存在: {d}")
            continue
        for p in d.rglob("*"):
            if p.is_file() and p.suffix.lower() in ALLOWED_EXT:
                if p.name in index:
                    dupes.add(p.name)
                index.setdefault(p.name, p)
    return index, dupes


def load_mappings():
    """读 mapping CSV: image_id -> (original_name, file_hash)"""
    mappings = {}
    for csv_path in MAPPING_CSVS:
        if not csv_path.exists():
            print(f"[警告] CSV 不存在: {csv_path}")
            continue
        with open(csv_path, newline="", encoding="utf-8") as f:
            for row in csv.DictReader(f):
                iid = row["image_id"].strip()
                if iid and iid not in mappings:
                    mappings[iid] = (row["original_name"].strip(), row["file_hash"].strip())
    return mappings


def load_db_dump():
    """读 DB 导出: image_id -> storage key (file_path)"""
    rows = {}
    with open(DB_DUMP, newline="", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            iid, key = line.split(",", 1)
            rows[iid.strip()] = key.strip()
    return rows


def upload_one(session, key, file_path, dry_run):
    if dry_run:
        return True, "dry-run"
    url = f"{BASE_URL}/restore/{key}"
    for attempt in range(3):
        try:
            with open(file_path, "rb") as fp:
                data = fp.read()
            r = session.put(
                url,
                data=data,
                headers={"X-Delete-Key": DELETE_KEY, "Content-Type": "application/octet-stream"},
                timeout=60,
            )
            if r.status_code in (200, 201):
                return True, "ok"
            if r.status_code in (401, 403):
                return False, f"鉴权失败 ({r.status_code})，停止重试"
        except requests.RequestException as e:
            err = str(e)
    return False, f"失败: {r.status_code if 'r' in dir() else ''} {err if 'err' in dir() else ''}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true", help="只核对，不上传")
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--retry-failed", action="store_true",
                    help="只重跑 restore_failed.csv 里的失败项")
    args = ap.parse_args()

    if not args.dry_run and not DELETE_KEY:
        print("错误: 请设置环境变量 RESTORE_DELETE_KEY")
        sys.exit(1)

    db_rows = load_db_dump()
    mappings = load_mappings()
    index, dupes = build_local_index()

    print(f"DB 记录: {len(db_rows)} 条")
    print(f"mapping CSV 记录: {len(mappings)} 条")
    print(f"本地原图: {len(index)} 个文件")
    if dupes:
        print(f"[警告] {len(dupes)} 个文件名在多个目录重复，将使用先扫到的: {sorted(dupes)[:5]}...")

    # 以 DB 为准做恢复计划
    plan = []          # (image_id, key, local_path)
    no_mapping = []    # DB 有记录但 CSV 没有（无法知道对应哪张原图）
    no_file = []       # CSV 有记录但本地找不到文件
    hash_bad = []      # 本地文件 MD5 与 CSV 记录不符

    for iid, key in db_rows.items():
        if iid not in mappings:
            no_mapping.append(iid)
            continue
        original_name, expected_hash = mappings[iid]
        local = index.get(original_name)
        if local is None:
            no_file.append((iid, original_name))
            continue
        if expected_hash and md5_of(local) != expected_hash:
            hash_bad.append((iid, original_name))
            continue
        plan.append((iid, key, local))

    if args.retry_failed:
        failed_csv = SCRIPT_DIR / "restore_failed.csv"
        with open(failed_csv, newline="", encoding="utf-8") as f:
            retry_ids = {row["image_id"].strip() for row in csv.DictReader(f)}
        plan = [p for p in plan if p[0] in retry_ids]
        print(f"只重跑失败项: {len(plan)} 张")

    print("-" * 50)
    print(f"可恢复: {len(plan)} 张")
    print(f"DB 有记录但无 mapping（跳过）: {len(no_mapping)} 条")
    print(f"本地缺文件: {len(no_file)} 条")
    print(f"MD5 不匹配: {len(hash_bad)} 条")

    if args.dry_run:
        print("\n[dry-run] 前 5 条计划:")
        for iid, key, local in plan[:5]:
            print(f"  {iid} -> {key}  <-  {local}")
        return

    ok, fail = 0, []
    session = requests.Session()
    with ThreadPoolExecutor(max_workers=args.workers) as ex:
        futures = {
            ex.submit(upload_one, session, key, local, False): (iid, key)
            for iid, key, local in plan
        }
        for n, fut in enumerate(as_completed(futures), 1):
            iid, key = futures[fut]
            success, msg = fut.result()
            if success:
                ok += 1
            else:
                fail.append((iid, key, msg))
            if n % 100 == 0 or n == len(plan):
                print(f"进度 {n}/{len(plan)}，成功 {ok}，失败 {len(fail)}")

    print("-" * 50)
    print(f"完成: 成功 {ok}，失败 {len(fail)}")
    if fail:
        out = SCRIPT_DIR / "restore_failed.csv"
        with open(out, "w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(["image_id", "key", "error"])
            w.writerows(fail)
        print(f"失败清单已写入: {out}")


if __name__ == "__main__":
    main()
