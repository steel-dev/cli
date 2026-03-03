export interface ActionPolicy {
    default: 'allow' | 'deny';
    allow?: string[];
    deny?: string[];
}
export type PolicyDecision = 'allow' | 'deny' | 'confirm';
export declare const KNOWN_CATEGORIES: Set<string>;
export declare function getActionCategory(action: string): string;
export declare function loadPolicyFile(policyPath: string): ActionPolicy;
export declare function initPolicyReloader(policyPath: string, policy: ActionPolicy): void;
export declare function reloadPolicyIfChanged(): ActionPolicy | null;
export declare function checkPolicy(action: string, policy: ActionPolicy | null, confirmCategories: Set<string>): PolicyDecision;
export declare function describeAction(action: string, command: Record<string, unknown>): string;
//# sourceMappingURL=action-policy.d.ts.map