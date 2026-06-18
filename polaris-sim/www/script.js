import init, { Simulation } from "../pkg/polaris_sim.js";
import { SimState, nodeStore, nodeRenderer } from "./nodeManager.js";

let sim = null;

async function run() {
    await init();
    sim = new Simulation();
    initEventListeners();
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", handleTickClick);

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", handleSpawnClick);

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("keydown", handleKeyDown);

    const canvas = document.getElementById("canvas");
    canvas.addEventListener("click", handleCanvasClick);

    document.body.addEventListener("click", handleDocumentClick);

    const info = document.getElementById("info-display");
    info.addEventListener("click", handleInfoClick);
}

function handleTickClick() {
    if (nodeStore.state === SimState.Spawning || nodeStore.state === SimState.SendingHello) {
        return;
    }

    sim.tick(10);

    if (nodeStore.state === SimState.Selected) {
        const json = sim.node_info(nodeStore.selectedNodeId);
        const data = JSON.parse(json);
        nodeRenderer.selectNode(nodeRenderer.selectedDom, data.id, data.connections);
    }
}

function handleSpawnClick() {
    if (nodeStore.state === SimState.Selected) {
        nodeStore.deselectNode();
        nodeRenderer.deselectNode();

    }

    if (nodeStore.state === SimState.Default) {
        nodeStore.startSpawning();
        nodeRenderer.spawnPreview();
    }

}

function handleMouseMove(e) {
    if (nodeStore.state === SimState.Spawning) {
        nodeRenderer.movePreview(e.clientX, e.clientY);
    }
}

function handleKeyDown(e) {
    if (e.key === "Escape") {
        if (nodeStore.state === SimState.Spawning) {
            nodeStore.cancelSpawning();
            nodeRenderer.removePreview();
        }
    }
}

function handleCanvasClick(e) {
    if (nodeStore.state === SimState.Spawning) {
        nodeStore.createNode();
        nodeRenderer.placePreview();
        sim.spawn_node();
    } else {
        nodeStore.deselectNode();
        nodeRenderer.deselectNode();
    }
}

function handleDocumentClick(e) {
    const node = e.target.closest(".node");
    if (!node) {
        return;
    }

    switch (nodeStore.state) {
        case SimState.Default:
            // fallthrough
        case SimState.Selected:
            const json = sim.node_info(node.dataset.id);
            const data = JSON.parse(json);
            nodeStore.selectNode(data.id);
            nodeRenderer.selectNode(node, data.id, data.connections);
            break;
        case SimState.SendingHello:
            sim.send_hello(nodeStore.selectedNodeId, node.dataset.id);
            nodeStore.endHello();
            nodeRenderer.endHello();
        case SimState.Spawning:
            break;
    }
}

function handleInfoClick(e) {
    if (nodeStore.state === SimState.Selected && e.target.id === "send") {
        nodeStore.startHello();
        nodeRenderer.startHello();
    }
}

run();
