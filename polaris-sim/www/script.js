import init, { Simulation } from "../pkg/polaris_sim.js";

let sim = null;
let newCircle = null;
let selectedCircle = null;
const circles = [];

async function run() {
    await init();
    sim = new Simulation();
    initEventListeners();
    console.log("Simulation created");
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", () => {
        sim.tick(10);
        console.log(sim.frame());
    });

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", createCircle);

    const canvas = document.getElementById("canvas");
    canvas.addEventListener("click", deselectCircle);
}

function createCircle() {
    if (newCircle) {
        return;
    }

    newCircle = document.createElement("div");
    newCircle.classList.add("circle");
    document.body.appendChild(newCircle);

    window.addEventListener("mousemove", moveCircle);
    window.addEventListener("click", dropCircle);
    window.addEventListener("keydown", cancelCircle);
}

function moveCircle(e) {
    if (!newCircle) {
        return;
    }

    newCircle.style.left = `${e.clientX}px`;
    newCircle.style.top = `${e.clientY}px`;
}

function dropCircle(e) {
    if (!newCircle) {
        return;
    }

    if (e.target.tagName == "DIV" || e.target.tagName == "BUTTON") {
        return;
    }

    newCircle.style.backgroundColor = "#cc5500";
    window.removeEventListener("mousemove", moveCircle);
    window.removeEventListener("click", dropCircle);
    window.removeEventListener("keydown", cancelCircle);

    const circle = newCircle;
    circle.dataset.id = circles.length;
    circle.classList.add("node");
    circles.push(circle);

    circle.addEventListener("click", () => displayNodeInfo(circle));

    sim.spawn_node();

    displayNodeInfo(circle);

    console.log("Node spawned");
    newCircle = null;
}

function cancelCircle(e) {
    if (!newCircle) {
        return;
    }

    if (e.key == "Escape") {
        newCircle.remove();
        window.removeEventListener("mousemove", moveCircle);
        window.removeEventListener("click", dropCircle);
        window.removeEventListener("keydown", cancelCircle);

        newCircle = null;
    }
}

function displayNodeInfo(circle) {
    const node = sim.node_info(circle.dataset.id);
    const data = JSON.parse(node);

    if (selectedCircle) {
        selectedCircle.style.backgroundColor = "#cc5500";
    }

    circle.style.backgroundColor = "#82af5f";
    selectedCircle = circle;

    const info = document.getElementById("info-display");
    let html = `<div class="inner-text">ID: ${data.id}</div>`;

    if (data.connections.length === 0) {
        html += '<div class="inner-text">Peers: None</div>';
    } else {
        const peers = data.connections.join(' ');
        html += `<div class="inner-text">Peers: ${peers}</div>`;
    }

    info.innerHTML = html;
}

function deselectCircle(e) {
    if (!selectedCircle) {
        return;
    }

    selectedCircle.style.backgroundColor = "#cc5500";
    selectedCircle = null;

    const info = document.getElementById("info-display");
    info.innerHTML = '<span class="placeholder-text">No node selected</span>';
}

run();
