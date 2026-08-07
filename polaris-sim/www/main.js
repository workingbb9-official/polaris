import init, { Simulation } from "../pkg/polaris_sim.js";
import { Mode, uiState } from "./uiState.js";
import { nodeRenderer } from "./nodeRenderer.js";
import { scriptRunner } from "./scriptRunner.js";

let sim = null;
let timerId = null;
let resizeTimer = null;

async function run() {
    await init();
    sim = new Simulation();
    nodeRenderer.resizeCanvas();
    initEventListeners();
}

function initEventListeners() {
    const tick = document.getElementById("tick");
    tick.addEventListener("click", handleTickClick);

    const spawn = document.getElementById("spawn");
    spawn.addEventListener("click", handleSpawnClick);

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", handleWindowResize)

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
    if (uiState.mode === Mode.Spawning || uiState.mode === Mode.SendingHello) {
        return;
    }

    if (timerId !== null) {
        nodeRenderer.stopTicking();
        clearInterval(timerId);
        timerId = null;
        return;
    }

    timerId = setInterval(() => {
        const events = JSON.parse(sim.tick(10));

        if (uiState.mode === Mode.Selected) {
            const json = sim.node_info(uiState.selectedNodeId);
            const data = JSON.parse(json);
            nodeRenderer.selectNode(nodeRenderer.selectedDom, data.id, data.uptime, data.peers);
        }

        handleSimEvents(events)
        uiState.increaseUptime(10);
        nodeRenderer.displayUptime(uiState.uptime);
    }, 1000);

    nodeRenderer.startTicking();
}

function handleSpawnClick() {
    if (uiState.mode === Mode.Selected) {
        uiState.deselectNode();
        nodeRenderer.deselectNode();

    }

    if (uiState.mode === Mode.Default) {
        uiState.startConfiguring();
        nodeRenderer.showConfigPage();
    }

}

function handleConfirmConfig() {
    if (uiState.mode !== Mode.Configuring) {
        return;
    }

    const raw = document.getElementById("node-heartbeat-input").value;
    const heartbeat = parseInt(raw, 10);

    if (isNaN(heartbeat) || heartbeat < 0) {
        document.getElementById("node-heartbeat-input").value = 1000;
        return;
    }

    uiState.startSpawning(heartbeat);
    nodeRenderer.spawnPreview();
    nodeRenderer.hideConfigPage();
}

function handleCancelConfig() {
    if (uiState.mode !== Mode.Configuring) {
        return;
    }

    uiState.cancelConfiguring();
    nodeRenderer.hideConfigPage();
}

function handleMouseMove(e) {
    if (uiState.mode === Mode.Spawning) {
        nodeRenderer.movePreview(e.clientX, e.clientY);
    }
}

function handleKeyDown(e) {
    if (e.key === "Escape") {
        if (uiState.mode === Mode.Spawning) {
            uiState.cancelSpawning();
            nodeRenderer.removePreview();
        }
    }
}

function handleCanvasClick(e) {
    if (uiState.mode === Mode.Spawning) {
        nodeRenderer.placePreview(sim.total_nodes());
        sim.spawn_node(e.clientX, e.clientY, uiState.currentConfig.heartbeat);
        uiState.createNode();
    } else {
        uiState.deselectNode();
        nodeRenderer.deselectNode();
    }
}

function handleDocumentClick(e) {
    const node = e.target.closest(".node");
    if (!node) {
        return;
    }

    switch (uiState.mode ) {
        case Mode.Default:
            // fallthrough
        case Mode.Selected:
            const nodeId = parseInt(node.dataset.id, 10);

            const json = sim.node_info(nodeId);
            const data = JSON.parse(json);

            uiState.selectNode(data.id);
            nodeRenderer.selectNode(node, data.id, data.uptime, data.peers);
            break;
        case Mode.SendingHello:
            const targetNodeId = parseInt(node.dataset.id, 10);

            sim.send_hello(uiState.selectedNodeId, targetNodeId);
            uiState.endHello();
            nodeRenderer.endHello();

            const [x1, y1] = sim.node_position(uiState.selectedNodeId);
            const [x2, y2] = sim.node_position(targetNodeId);

            nodeRenderer.sendPacket(x1, y1, x2, y2);
            break;
        case Mode.Spawning:
            break;
    }
}

function handleInfoClick(e) {
    if (uiState.mode !== Mode.Selected) {
        return;
    }

    if (e.target.id === "send") {
        uiState.startHello();
        nodeRenderer.startHello();
    }

    if (e.target.id === "submit-script-btn") {
        const text = document.getElementById("node-script-input").value.trim();
        console.log(`Script submitted: ${text}`);
        scriptRunner.runNodeScript(text);
    }
}

function handleSimEvents(events) {
    for (const event of events) {
        switch (event.type) {
            case "WelcomePacketSent":
                nodeRenderer.sendPacket(
                    event.from_x,
                    event.from_y,
                    event.to_x,
                    event.to_y,
                    () => nodeRenderer.drawLineBetween(
                        event.from_x,
                        event.from_y,
                        event.to_x,
                        event.to_y,
                    ),
                );
                break;
            case "HeartbeatPacketSent":
                nodeRenderer.sendPacket(
                    event.from_x,
                    event.from_y,
                    event.to_x,
                    event.to_y,
                );
                break;
            default:
                break;
        }
    }
}

function handleWindowResize() {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
        nodeRenderer.redrawLines();
        nodeRenderer.repositionNodes();
    }, 100);
}

run();
