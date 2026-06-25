export const Mode = {
    Default: "default",
    Configuring: "configuring",
    Spawning: "spawning",
    Selected: "selected",
    SendingHello: "sending hello",
};

export const uiState = {
    uptime: 0,
    mode: Mode.Default,
    selectedNodeId: null,
    currentConfig: {
        heartbeat: null,
    },

    startConfiguring() {
        this.currentConfig.heartbeat = null;
        this.mode = Mode.Configuring;
    },

    cancelConfiguring() {
        this.mode = Mode.Default;
    },

    startSpawning(heartbeat) {
        this.currentConfig.heartbeat = heartbeat;
        this.mode = Mode.Spawning;
    },

    createNode() {
        this.mode = Mode.Default;
    },

    cancelSpawning() {
        this.mode = Mode.Default;
    },

    selectNode(id) {
        this.selectedNodeId = id;
        this.mode = Mode.Selected;
    },

    deselectNode() {
        this.selectedNodeId = null;
        this.mode = Mode.Default;
    },

    startHello() {
        this.mode = Mode.SendingHello;
    },

    endHello() {
        this.mode = Mode.Selected;
    },

    increaseUptime(elapsed) {
        this.uptime += elapsed;
    },
};
