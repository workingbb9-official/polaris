export const SimState = {
    Default: "default",
    Configuring: "configuring",
    Spawning: "spawning",
    Selected: "selected",
    SendingHello: "sending hello",
};

export const nodeStore = {
    state: SimState.Default,
    selectedNodeId: null,
    currentConfig: {
        heartbeat: null,
    },

    startConfiguring() {
        this.currentConfig.heartbeat = null;
        this.state = SimState.Configuring;
    },

    cancelConfiguring() {
        this.state = SimState.Default;
    },

    startSpawning(heartbeat) {
        this.currentConfig.heartbeat = heartbeat;
        this.state = SimState.Spawning;
    },

    createNode() {
        this.state = SimState.Default;
    },

    cancelSpawning() {
        this.state = SimState.Default;
    },

    selectNode(id) {
        this.selectedNodeId = id;
        this.state = SimState.Selected;
    },

    deselectNode() {
        this.selectedNodeId = null;
        this.state = SimState.Default;
    },

    startHello() {
        this.state = SimState.SendingHello;
    },

    endHello() {
        this.state = SimState.Selected;
    },
};
