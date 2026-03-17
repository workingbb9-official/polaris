// static/script.js
document.addEventListener('DOMContentLoaded', () => {
    const statusSpan = document.querySelector('#status span');
    const testBtn = document.getElementById('testBtn');

    // 1. Immediate feedback that JS loaded
    statusSpan.textContent = "Active";
    statusSpan.className = "connected";
    console.log("Polaris test script initialized.");

    // 2. Interaction test
    testBtn.addEventListener('click', () => {
        alert("JavaScript is working! Your Rust server served this file successfully.");
        
        // Let's do something fun to the UI
        document.querySelector('.container').style.borderColor = '#10b981';
        testBtn.textContent = "Test Successful!";
        testBtn.style.backgroundColor = "#10b981";
    });
});
