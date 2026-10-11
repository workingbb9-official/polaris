export const scriptRunner = {
    scripts: new Map(),

    setNodeScript(nodeId, userCode) {
        this.scripts.set(nodeId, userCode);
    },

    getNodeScript(nodeId) {
        return this.scripts.get(nodeId) || null;
    },

    runNodeScript(nodeId, nodeInfo) {
        const userCode = this.scripts.get(nodeId);
        if (!userCode) {
            console.log(`No script found for node ${nodeId}`);
            return false;
        }

        const nodeAPI = {
            info: Object.freeze({
                id: nodeInfo.id,
                uptime: nodeInfo.uptime,
                peers: [...nodeInfo.peers]
            }),
        };

        try {
            const runner = new Function("node", userCode);
            runner(nodeAPI);
            return true;
        } catch (error) {
            console.error(`User code crashed for node ${nodeId}:`, error.message);
            return false;
        }
    },

    runScripts(getNodeInfo) {
        for (const nodeId of this.scripts.keys()) {
            const nodeInfo = getNodeInfo(nodeId);
            this.runNodeScript(nodeId, nodeInfo);
        }
    },
};
