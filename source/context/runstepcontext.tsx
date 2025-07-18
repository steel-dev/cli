import React, {createContext, useContext, useState, ReactNode} from 'react';
import {Template} from '../utils/types.js';

type RunStepContextType = {
	step: string;
	setStep: (step: string) => void;
	template: Template | null;
	setTemplate: (template: Template) => void;
	envVars: Record<string, string> | null;
	setEnvVars: (envVars: Record<string, string> | null) => void;
	directory: string | null;
	setDirectory: (directory: string | null) => void;
	sessionId: string | null;
	setSessionId: (id: string | null) => void;
	hash: string | null;
	setHash: (hash: string | null) => void;
};

const RunStepContext = createContext<RunStepContextType | undefined>(undefined);

export const RunStepProvider = ({children}: {children: ReactNode}) => {
	// For run command, start at template step instead of projectname
	const [step, setStep] = useState<string>('template');
	const [template, setTemplate] = useState<Template | null>(null);
	const [envVars, setEnvVars] = useState<Record<string, string>>({});
	const [directory, setDirectory] = useState<string | null>(null);
	const [sessionId, setSessionId] = useState<string | null>(null);
	const [hash, setHash] = useState<string | null>(null);

	return (
		<RunStepContext.Provider
			value={{
				step,
				setStep,
				template,
				setTemplate,
				envVars,
				setEnvVars,
				directory,
				setDirectory,
				sessionId,
				setSessionId,
				hash,
				setHash,
			}}
		>
			{children}
		</RunStepContext.Provider>
	);
};

export const useRunStep = () => {
	const context = useContext(RunStepContext);
	if (!context)
		throw new Error('useRunStep must be used within a RunStepProvider');
	return context;
};
