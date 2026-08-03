export const nodeRenderer = {
    previewDom: null,
    selectedDom: null,
    lines: [],

    showConfigPage() {
        const info = document.getElementById("info-section");
        info.classList.add("hidden");

        const config = document.getElementById("config-section");
        config.classList.remove("hidden");
    },

    hideConfigPage() {
        const config = document.getElementById("config-section");
        config.classList.add("hidden");

        const info = document.getElementById("info-section");
        info.classList.remove("hidden");
    },

    spawnPreview() {
        const node = document.createElement("div");
        node.classList.add("node", "preview");
        document.body.appendChild(node);
        this.previewDom = node;
    },

    movePreview(clientX, clientY) {
        this.previewDom.style.left = `${clientX}px`;
        this.previewDom.style.top = `${clientY}px`;
    },

    placePreview(id) {
        const node = this.previewDom;
        node.classList.remove("preview");

        node.dataset.id = id;
        this.pendingNode = null;
    },

    removePreview() {
        this.previewDom.remove();
        this.previewDom = null;
    },

    selectNode(node, id, uptime, peers) {
        if (this.selectedDom) {
            this.selectedDom.classList.remove("selected");
        }

        const info = document.getElementById("info-display");
        let html = `<div class="inner-text">ID: ${id}</div>`;
        html += `<div class="inner-text">Uptime: ${uptime}</div>`;

        if (peers.length === 0) {
            html += '<div class="inner-text">Peers: None</div>';
        } else if (peers.length === 1) {
            const peer = peers[0];
            html += `<div class="inner-text">Peer: Node ${peer}</div>`;
        } else {
            const peer_list = peers.join(", ");
            html += `<div class="inner-text">Peers: Nodes ${peer_list}</div>`;
        }

        html += '<button id="send" class="btn btn-primary">Send Hello</button>';

        html += `
            <div id="script-section">\
                <h3 class="inner-text">Custom Script</h3>\
                <textarea id="node-script-input" rows="5"></textarea>\
                <button type="button" id="submit-script-btn" class="btn btn-primary">\
                    Embed Script\
                </button>\
            </div>
        `;

        node.classList.add("selected");
        this.selectedDom = node;
        info.innerHTML = html;
    },

    deselectNode() {
        if (this.selectedDom) {
            this.selectedDom.classList.remove("selected");
        }

        this.selectedDom = null;

        const instruction = document.getElementById("select-node");
        instruction.classList.remove("show");

        const info = document.getElementById("info-display");
        info.innerHTML = '<span class="placeholder-text">No node selected</span>';
    },

    startHello() {
        const instruction = document.getElementById("select-node");
        instruction.classList.add("show");
    },

    endHello() {
        const instruction = document.getElementById("select-node");
        instruction.classList.remove("show");
    },

    resizeCanvas() {
        const canvas = document.getElementById("canvas");
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    },

    renderSingleLine(x1, y1, x2, y2) {
        const canvas = document.getElementById("canvas");
        const ctx = canvas.getContext("2d");

        // Draw relative to canvas and not window
        const rect = canvas.getBoundingClientRect();
        ctx.beginPath();
        ctx.moveTo(x1 - rect.left, y1 - rect.top);
        ctx.lineTo(x2 - rect.left, y2 - rect.top);

        ctx.strokeStyle = "#2ecc71";
        ctx.lineWidth = 4;
        ctx.lineCap = "round";

        ctx.stroke();
    },

    drawLineBetween(x1, y1, x2, y2) {
        this.lines.push({x1, y1, x2, y2})
        this.renderSingleLine(x1, y1, x2, y2)
    },

    redrawLines() {
        const canvas = document.getElementById("canvas");
        const ctx = canvas.getContext("2d");

        ctx.clearRect(0, 0, canvas.width, canvas.height);

        for (const line of this.lines) {
            this.renderSingleLine(line.x1, line.y1, line.x2, line.y2);
        }
    },

    sendPacket(x1, y1, x2, y2, onArrive) {
        const packet = packetImg.cloneNode();

        packet.style.cssText = `
            position: fixed;
            z-index: 9999;
            pointer-events: none;
            left: ${x1}px;
            top:  ${y1}px;
            transform: translate(-50%, -50%);
            transition: left 1000ms linear, top 1000ms linear;
          `;

        document.body.appendChild(packet);

        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                packet.style.left = `${x2}px`;
                packet.style.top  = `${y2}px`;
            });
        });

        packet.addEventListener("transitionend", () => {
            packet.remove();
            if (onArrive) {
                onArrive();
            }
        }, { once: true });
    },

    displayUptime(uptime) {
        const box = document.getElementById("time-box");
        box.textContent = uptime;
    },

    startTicking() {
        const tickButton = document.getElementById("tick");
        tickButton.innerText = "Stop Ticking";
    },

    stopTicking() {
        const tickButton = document.getElementById("tick");
        tickButton.innerText = "Start Ticking";
    }
};

const packetSVG = `
  <svg xmlns="http://www.w3.org/2000/svg" width="48" height="54" viewBox="0 0 32 40">
    <path d="M4,0 L22,0 L32,10 L32,40 L4,40 Z" fill="#f0f4ff" stroke="#4A90D9" stroke-width="2"/>
    <path d="M22,0 L22,10 L32,10" fill="#c0d0f0" stroke="#4A90D9" stroke-width="2"/>
    <line x1="9" y1="18" x2="26" y2="18" stroke="#4A90D9" stroke-width="1.5" stroke-linecap="round"/>
    <line x1="9" y1="24" x2="26" y2="24" stroke="#4A90D9" stroke-width="1.5" stroke-linecap="round"/>
    <line x1="9" y1="30" x2="20" y2="30" stroke="#4A90D9" stroke-width="1.5" stroke-linecap="round"/>
  </svg>
`;

const packetImg = svgToImg(packetSVG);

function svgToImg(svgString) {
    const blob = new Blob([svgString], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    const img = new Image();
    img.src = url;

    return img;
}
