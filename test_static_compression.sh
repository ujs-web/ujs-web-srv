#!/bin/bash

echo "测试静态资源服务（带压缩和缓存）"
echo "=================================="

BASE_URL="http://localhost:3001"

echo -e "\n1. 测试根路径访问（应返回 index.html）:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" $BASE_URL/

echo -e "\n2. 测试 /static 路径访问:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" $BASE_URL/static/index.html

echo -e "\n3. 测试 /assets 路径访问:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" $BASE_URL/assets/css/style.css

echo -e "\n4. 测试 /images 路径访问（假设有图片）:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" $BASE_URL/images/test.jpg 2>&1

echo -e "\n5. 测试压缩支持（Accept-Encoding: gzip）:"
curl -s -I -H "Accept-Encoding: gzip, deflate, br" $BASE_URL/static/index.html | grep -i "content-encoding"

echo -e "\n6. 测试缓存头（Cache-Control）:"
curl -s -I $BASE_URL/static/css/style.css | grep -i "cache-control"

echo -e "\n7. 测试 CORS 头:"
curl -s -I -H "Origin: http://example.com" $BASE_URL/static/index.html | grep -i "access-control-allow-origin"

echo -e "\n8. 测试不存在的路径（应返回 index.html）:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" $BASE_URL/nonexistent

echo -e "\n测试完成！"