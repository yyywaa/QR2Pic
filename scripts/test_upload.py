#!/usr/bin/env python3
"""
简单的API测试脚本
测试图片上传和获取功能
"""

import os
import sys
import requests
from pathlib import Path

API_BASE = "http://cccf.zeabur.app"

def test_upload():
    """测试图片上传"""
    test_image = Path("test/70154AA03257AA.jpg")
    
    if not test_image.exists():
        print(f"错误: 测试图片不存在 - {test_image}")
        return None
    
    print(f"上传图片: {test_image.name} ({test_image.stat().st_size} 字节)")
    
    try:
        with open(test_image, 'rb') as f:
            files = {'file': (test_image.name, f, 'image/jpeg')}
            response = requests.post(f"{API_BASE}/upload", files=files, timeout=30, verify=False)
        
        print(f"状态码: {response.status_code}")
        print(f"响应头: {dict(response.headers)}")
        
        if response.status_code == 200:
            result = response.json()
            print(f"上传成功:")
            print(f"  ID: {result.get('id')}")
            print(f"  URL: {result.get('url')}")
            return result
        else:
            print(f"上传失败: {response.text}")
            return None
            
    except Exception as e:
        print(f"请求异常: {e}")
        return None

def test_get_image(image_id):
    """测试获取图片（重定向）"""
    print(f"\n测试获取图片: {image_id}")
    
    try:
        # 使用 HEAD 请求查看重定向
        response = requests.head(f"{API_BASE}/image/{image_id}", timeout=30, allow_redirects=False, verify=False)
        
        print(f"状态码: {response.status_code}")
        
        if response.status_code == 302:
            redirect_url = response.headers.get('Location')
            print(f"重定向到: {redirect_url}")
            
            # 尝试访问重定向的URL
            if redirect_url:
                print(f"访问重定向URL...")
                img_response = requests.get(redirect_url, timeout=30, verify=False)
                print(f"图片访问状态码: {img_response.status_code}")
                print(f"图片大小: {len(img_response.content)} 字节" if img_response.status_code == 200 else "图片访问失败")
        else:
            print(f"响应头: {dict(response.headers)}")
            
    except Exception as e:
        print(f"请求异常: {e}")

def test_health():
    """测试健康检查"""
    print("测试健康检查...")
    
    try:
        response = requests.get(f"{API_BASE}/health", timeout=10, verify=False)
        print(f"状态码: {response.status_code}")
        print(f"响应: {response.text}")
    except Exception as e:
        print(f"健康检查失败: {e}")

def test_options():
    """测试OPTIONS方法"""
    print("测试OPTIONS方法...")
    
    try:
        response = requests.options(f"{API_BASE}/upload", timeout=10, verify=False)
        print(f"状态码: {response.status_code}")
        print(f"Allow头: {response.headers.get('Allow')}")
        print(f"响应头: {dict(response.headers)}")
    except Exception as e:
        print(f"OPTIONS测试失败: {e}")

def main():
    print(f"测试 API: {API_BASE}")
    print("=" * 50)
    
    # 测试健康检查
    test_health()
    print("-" * 50)
    
    # 测试OPTIONS
    test_options()
    print("-" * 50)
    
    # 测试上传
    result = test_upload()
    
    if result:
        # 测试获取
        test_get_image(result['id'])
    
    print("=" * 50)
    print("测试完成")

if __name__ == '__main__':
    main()