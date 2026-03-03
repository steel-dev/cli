export declare function requestConfirmation(action: string, category: string, description: string, command: Record<string, unknown>): {
    confirmationId: string;
};
export declare function getAndRemovePending(id: string): {
    command: Record<string, unknown>;
    action: string;
} | null;
//# sourceMappingURL=confirmation.d.ts.map