interface AuthProfile {
    name: string;
    url: string;
    username: string;
    password: string;
    usernameSelector?: string;
    passwordSelector?: string;
    submitSelector?: string;
    createdAt: string;
    lastLoginAt?: string;
}
export interface AuthProfileMeta {
    name: string;
    url: string;
    username: string;
    createdAt: string;
    lastLoginAt?: string;
}
export declare function saveAuthProfile(opts: {
    name: string;
    url: string;
    username: string;
    password: string;
    usernameSelector?: string;
    passwordSelector?: string;
    submitSelector?: string;
}): AuthProfileMeta & {
    updated: boolean;
};
export declare function getAuthProfile(name: string): AuthProfile | null;
export declare function getAuthProfileMeta(name: string): AuthProfileMeta | null;
export declare function listAuthProfiles(): AuthProfileMeta[];
export declare function deleteAuthProfile(name: string): boolean;
export declare function updateLastLogin(name: string): void;
export {};
//# sourceMappingURL=auth-vault.d.ts.map