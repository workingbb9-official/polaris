import init, { Simulation } from "../pkg/polaris_sim.js";

let sim = null;
let newNode = null;
let selectedNode = null;
const nodes = [];

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
    spawn.addEventListener("click", createNode);

    const canvas = document.getElementById("canvas");
    canvas.addEventListener("click", deselectNode);

    document.body.addEventListener("click", (e) => {
        const clickedNode = e.target.closest(".node");
        if (clickedNode) {
            displayNodeInfo(clickedNode);
        }
    });

    const info = document.getElementById("info-display");
    info.addEventListener("click", onInfoClick);
}

function createNode() {
    if (newNode) {
        return;
    }

    newNode = document.createElement("div");
    newNode.classList.add("node", "preview");
    document.body.appendChild(newNode);

    window.addEventListener("mousemove", moveNode);
    window.addEventListener("click", dropNode);
    window.addEventListener("keydown", cancelNode);
}

function moveNode(e) {
    if (!newNode) {
        return;
    }

    newNode.style.left = `${e.clientX}px`;
    newNode.style.top = `${e.clientY}px`;
}

function dropNode(e) {
    if (!newNode) {
        return;
    }

    if (e.target.tagName === "DIV" || e.target.tagName === "BUTTON") {
        return;
    }

    window.removeEventListener("mousemove", moveNode);
    window.removeEventListener("click", dropNode);
    window.removeEventListener("keydown", cancelNode);

    const node = newNode;
    node.classList.remove("preview");
    node.dataset.id = nodes.length;
    nodes.push(node);

    console.log("Node spawned");
    sim.spawn_node();
    newNode = null;
}

function cancelNode(e) {
    if (!newNode) {
        return;
    }

    if (e.key === "Escape") {
        newNode.remove();
        window.removeEventListener("mousemove", moveNode);
        window.removeEventListener("click", dropNode);
        window.removeEventListener("keydown", cancelNode);

        newNode = null;
    }
}

function displayNodeInfo(node) {
    const json = sim.node_info(node.dataset.id);
    const data = JSON.parse(json);

    if (selectedNode) {
        selectedNode.classList.remove("selected");
    }

    node.classList.add("selected");
    selectedNode = node;

    const info = document.getElementById("info-display");
    let html = `<div class="inner-text">ID: ${data.id}</div>`;

    if (data.connections.length === 0) {
        html += '<div class="inner-text">Peers: None</div>';
    } else {
        const peers = data.connections.join(' ');
        html += `<div class="inner-text">Peers: ${peers}</div>`;
    }

    html += '<button id="send" class="btn btn-primary">Send Hello</button>';
    info.innerHTML = html;
}

function deselectNode(e) {
    if (!selectedNode || e.target.closest(".node")) {
        return;
    }

    selectedNode.classList.remove("selected");
    selectedNode = null;

    const instruction = document.getElementById("select-node");
    instruction.classList.remove("show");

    const info = document.getElementById("info-display");
    info.innerHTML = '<span class="placeholder-text">No node selected</span>';
}

function onInfoClick(e) {
    if (e.target.id === "send" && selectedNode) {
        console.log("Displaying instruction");
        const instruction = document.getElementById("select-node");
        instruction.classList.add("show");
        sendHello();
    }
}

function sendHello() {
    if (!selectedNode) {
        return;
    }
}

run();
