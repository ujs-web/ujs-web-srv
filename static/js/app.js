console.log('JavaScript 文件加载成功');

document.addEventListener('DOMContentLoaded', function() {
    const resultDiv = document.getElementById('js-result');
    if (resultDiv) {
        resultDiv.textContent = '✓ JavaScript 功能正常工作！';
        console.log('DOM 加载完成，JavaScript 已执行');
    }
});