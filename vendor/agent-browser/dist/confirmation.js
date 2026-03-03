import { randomBytes } from 'node:crypto';
const AUTO_DENY_TIMEOUT_MS = 60_000;
const pending = new Map();
function generateId() {
    return `c_${randomBytes(8).toString('hex')}`;
}
export function requestConfirmation(action, category, description, command) {
    const id = generateId();
    const timer = setTimeout(() => {
        pending.delete(id);
    }, AUTO_DENY_TIMEOUT_MS);
    pending.set(id, {
        id,
        action,
        category,
        description,
        command,
        timer,
    });
    return { confirmationId: id };
}
export function getAndRemovePending(id) {
    const entry = pending.get(id);
    if (!entry)
        return null;
    clearTimeout(entry.timer);
    pending.delete(id);
    return { command: entry.command, action: entry.action };
}
//# sourceMappingURL=confirmation.js.map