document.addEventListener('DOMContentLoaded', () => {
    const backBtn = document.getElementById('backBtn');

    console.log("Polaris About page initialized.");

    backBtn.addEventListener('click', () => {
        // Visual feedback
        backBtn.textContent = "Loading Home...";

        document.querySelector('.container').style.borderColor = '#10b981';
        backBtn.textContent = "Redirecting...";
        backBtn.style.backgroundColor = "#10b981";

        // 3. Move the URL to the / page
        // We use a small timeout so the user actually sees the button change 
        // before the page vanishes.
        setTimeout(() => {
            window.location.href = '/';
        }, 500); 

    });
});
