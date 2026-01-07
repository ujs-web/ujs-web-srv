#!/bin/bash

echo "测试静态资源访问功能"
echo "===================="

# 测试根路径（应该返回 index.html）
echo -e "\n1. 测试根路径访问 (http://localhost:3001/):"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/

# 测试静态资源路径
echo -e "\n2. 测试 /static/index.html 访问:"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/static/index.html

# 测试 CSS 文件
echo -e "\n3. 测试 CSS 文件访问 (http://localhost:3001/static/css/style.css):"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/static/css/style.css

# 测试 JS 文件
echo -e "\n4. 测试 JavaScript 文件访问 (http://localhost:3001/static/js/app.js):"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/static/js/app.js

# 测试不存在的路径（应该返回 index.html）
echo -e "\n5. 测试不存在的路径 (http://localhost:3001/nonexistent):"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/nonexistent

# 测试静态资源下的不存在的路径（应该返回 index.html）
echo -e "\n6. 测试 /static 下的不存在的路径 (http://localhost:3001/static/nonexistent):"
curl -s -o /dev/null -w "状态码: %{http_code}\n" http://localhost:3001/static/nonexistent

echo -e "\n测试完成！"