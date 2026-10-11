export const scriptRunner = {
    scripts: new Map(),

    setNodeScript(nodeId, userCode) {
        this.scripts.set(nodeId, userCode);
    },

    getNodeScript(nodeId) {
        return this.scripts.get(nodeId) || null;
    },

    runNodeScript(nodeId) {
        const userCode = this.scripts.get(nodeId);
        if (!userCode) {
            console.log(`No script found for node ${nodeId}`);
            return false;
        }

        try {
            const runner = new Function(userCode);
            runner();
            return true;
        } catch (error) {
            console.error(`User code crashed for node ${nodeId}:`, error.message);
            return false;
        }
    },

    runScripts() {
        for (const nodeId of this.scripts.keys()) {
            this.runNodeScript(nodeId);
        }
    },
};
