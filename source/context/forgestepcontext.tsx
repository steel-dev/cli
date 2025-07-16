import React, {createContext, useContext, useState, ReactNode} from 'react';
import {Template} from '../utils/types.js';

type ForgeStepContextType = {
	step: string;
	setStep: (step: string) => void;
	template: Template | null;
	setTemplate: (template: Template) => void;
	envVars: Record<string, string>;
	setEnvVars: (envVars: Record<string, string>) => void;
	directory: string;
	setDirectory: (dir: string) => void;
	packageManager: string;
	setPackageManager: (manager: string) => void;
	sessionId: string | null;
	setSessionId: (sessionId: string | null) => void;
};

const ForgeStepContext = createContext<ForgeStepContextType | undefined>(
	undefined,
);

export const ForgeStepProvider = ({children}: {children: ReactNode}) => {
	const [step, setStep] = useState<string>('projectname');
	const [template, setTemplate] = useState<Template | null>(null);
	const [directory, setDirectory] = useState<string>('steel-project');
	const [envVars, setEnvVars] = useState<Record<string, string>>({});
	const [packageManager, setPackageManager] = useState<string>('npm');
	const [sessionId, setSessionId] = useState<string | null>(null);

	return (
		<ForgeStepContext.Provider
			value={{
				step,
				setStep,
				template,
				setTemplate,
				envVars,
				setEnvVars,
				directory,
				setDirectory,
				packageManager,
				setPackageManager,
				sessionId,
				setSessionId,
			}}
		>
			{children}
		</ForgeStepContext.Provider>
	);
};

export const useForgeStep = () => {
	const context = useContext(ForgeStepContext);
	if (!context)
		throw new Error('useForgeStep must be used within a ForgeStepProvider');
	return context;
};
