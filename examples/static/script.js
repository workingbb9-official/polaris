document.addEventListener('DOMContentLoaded', () => {
    const statusSpan = document.querySelector('#status span');
    const testBtn = document.getElementById('testBtn');
    const sendBtn = document.getElementById('sendBtn');

    // 1. Immediate feedback that JS loaded
    statusSpan.textContent = "Active";
    statusSpan.className = "connected";
    console.log("Polaris test script initialized.");

    // 2. Interaction test
    testBtn.addEventListener('click', () => {
        // Change the UI first so the user sees a "flash" of success
        document.querySelector('.container').style.borderColor = '#10b981';
        testBtn.textContent = "Redirecting...";
        testBtn.style.backgroundColor = "#10b981";

        // 3. Move the URL to the /about page
        // We use a small timeout so the user actually sees the button change 
        // before the page vanishes.
        setTimeout(() => {
            window.location.href = '/about';
        }, 500); 
    });

    sendBtn.addEventListener('click', () => {
        window.location.href = '/post';
    });
});
