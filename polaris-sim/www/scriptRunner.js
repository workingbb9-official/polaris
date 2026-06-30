export const scriptRunner = {
    runNodeScript(userCode) {
        try {
            const runner = new Function(userCode);
            runner();
            return true;
        } catch (error) {
            console.error("User code crashed:", error.message);
            return false;
        }
    },
};
