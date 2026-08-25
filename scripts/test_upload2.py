#!/usr/bin/env python3
"""
测试API上传功能 - 使用HTTPS和重定向
"""

import requests
from pathlib import Path
import sys

API_BASE = "https://cccf.zeabur.app"

def create_session():
    """创建配置好的请求会话"""
    session = requests.Session()
    session.verify = False  # 忽略SSL证书验证
    session.allow_redirects = True  # 允许重定向
    session.headers.update({
        'User-Agent': 'QR2Pic-Test/1.0',
        'Accept': 'application/json',
    })
    # 禁用SSL警告
    import urllib3
    urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)
    return session

def test_health(session):
    """测试健康检查"""
    print("测试健康检查...")
    try:
        response = session.get(f"{API_BASE}/health", timeout=10)
        print(f"状态码: {response.status_code}")
        print(f"响应: {response.text}")
        return response.status_code == 200
    except Exception as e:
        print(f"健康检查失败: {e}")
        return False

def test_upload(session):
    """测试图片上传"""
    test_image = Path("test/70154AA03257AA.jpg")
    
    if not test_image.exists():
        print(f"错误: 测试图片不存在 - {test_image}")
        return None
    
    print(f"上传图片: {test_image.name} ({test_image.stat().st_size} 字节)")
    
    try:
        with open(test_image, 'rb') as f:
            files = {'file': (test_image.name, f, 'image/jpeg')}
            response = session.post(f"{API_BASE}/upload", files=files, timeout=30)
        
        print(f"状态码: {response.status_code}")
        print(f"响应头: {dict(response.headers)}")
        
        if response.status_code == 200:
            result = response.json()
            print(f"上传成功:")
            print(f"  ID: {result.get('id')}")
            print(f"  URL: {result.get('url')}")
            return result
        else:
            print(f"响应内容: {response.text[:200]}")
            return None
            
    except Exception as e:
        print(f"请求异常: {e}")
        import traceback
        traceback.print_exc()
        return None

def test_get_image(session, image_id):
    """测试获取图片（重定向）"""
    print(f"\n测试获取图片: {image_id}")
    
    try:
        # 首先尝试HEAD请求（不跟随重定向）
        response = session.head(f"{API_BASE}/image/{image_id}", timeout=30, allow_redirects=False)
        
        print(f"状态码: {response.status_code}")
        print(f"响应头: {dict(response.headers)}")
        
        if response.status_code == 302:
            redirect_url = response.headers.get('Location')
            print(f"重定向到: {redirect_url}")
            
            # 尝试访问重定向的URL
            if redirect_url:
                print(f"访问重定向URL...")
                img_response = session.get(redirect_url, timeout=30, allow_redirects=True)
                print(f"图片访问状态码: {img_response.status_code}")
                if img_response.status_code == 200:
                    print(f"图片大小: {len(img_response.content)} 字节")
                    print(f"Content-Type: {img_response.headers.get('Content-Type')}")
                else:
                    print(f"图片访问失败")
        else:
            print(f"未重定向")
            
    except Exception as e:
        print(f"请求异常: {e}")
        import traceback
        traceback.print_exc()

def main():
    print(f"测试 API: {API_BASE}")
    print("=" * 60)
    
    session = create_session()
    
    # 测试健康检查
    if not test_health(session):
        print("健康检查失败，退出测试")
        return
    
    print("-" * 60)
    
    # 测试上传
    result = test_upload(session)
    
    if result:
        # 测试获取
        test_get_image(session, result['id'])
    
    print("=" * 60)
    print("测试完成")

if __name__ == '__main__':
    main()