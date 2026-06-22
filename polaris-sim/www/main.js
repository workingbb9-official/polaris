import init, { Simulation } from "../pkg/polaris_sim.js";
import { SimState, nodeStore } from "./nodeStore.js";
import { nodeRenderer } from "./nodeRenderer.js"

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

    const confirmConfig = document.getElementById("confirm-config-btn");
    confirmConfig.addEventListener("click", handleConfirmConfig);

    const cancelConfig = document.getElementById("cancel-config-btn");
    cancelConfig.addEventListener("click", handleCancelConfig);
}

function handleTickClick() {
    if (nodeStore.state === SimState.Spawning || nodeStore.state === SimState.SendingHello) {
        return;
    }

    sim.tick(10);

    if (nodeStore.state === SimState.Selected) {
        const json = sim.node_info(nodeStore.selectedNodeId);
        const data = JSON.parse(json);
        nodeRenderer.selectNode(nodeRenderer.selectedDom, data.id, data.uptime, data.peers);
    }

    drawConnections();
}

function handleSpawnClick() {
    if (nodeStore.state === SimState.Selected) {
        nodeStore.deselectNode();
        nodeRenderer.deselectNode();

    }

    if (nodeStore.state === SimState.Default) {
        nodeStore.startConfiguring();
        nodeRenderer.showConfigPage();
    }

}

function handleConfirmConfig() {
    if (nodeStore.state !== SimState.Configuring) {
        return;
    }

    const raw = document.getElementById("node-heartbeat-input").value;
    const heartbeat = parseInt(raw, 10);

    if (isNaN(heartbeat) || heartbeat < 0) {
        document.getElementById("node-heartbeat-input").value = 1000;
        return;
    }

    nodeStore.startSpawning(heartbeat);
    nodeRenderer.spawnPreview();
    nodeRenderer.hideConfigPage();
}

function handleCancelConfig() {
    if (nodeStore.state !== SimState.Configuring) {
        return;
    }

    nodeStore.cancelConfiguring();
    nodeRenderer.hideConfigPage();
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
        nodeRenderer.placePreview(nodeStore.nodes.length);
        sim.spawn_node(nodeStore.currentConfig.heartbeat);

        nodeStore.createNode(e.clientX, e.clientY);
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
            const nodeId = parseInt(node.dataset.id, 10);

            const json = sim.node_info(nodeId);
            const data = JSON.parse(json);

            nodeStore.selectNode(data.id);
            nodeRenderer.selectNode(node, data.id, data.uptime, data.peers);
            break;
        case SimState.SendingHello:
            const targetNodeId = parseInt(node.dataset.id, 10);

            sim.send_hello(nodeStore.selectedNodeId, targetNodeId);
            nodeStore.endHello();
            nodeRenderer.endHello();
            break;
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

function drawConnections() {
    nodeRenderer.clearCanvas();

    const drawnConnections = new Set();

    for (const node of nodeStore.nodes) {
        const json = sim.node_info(node.id);
        const data = JSON.parse(json);

        for (const peerId of data.peers) {
            const pairKey = [node.id, peerId].sort().join("-");
            if (drawnConnections.has(pairKey)) {
                continue;
            }

            const peerJson = sim.node_info(peerId);
            const peerData = JSON.parse(peerJson);

            if (peerData.peers.includes(node.id)) {
                connectNodes(node.id, peerId);
                drawnConnections.add(pairKey);
            }
        }
    }
}

function connectNodes(node1, node2) {
    const firstNode = nodeStore.getNodeFromId(node1);
    const secondNode = nodeStore.getNodeFromId(node2);

    if (!firstNode || !secondNode) {
        return;
    }

    nodeRenderer.drawLineBetween(firstNode.x, firstNode.y, secondNode.x, secondNode.y);
}

run();
