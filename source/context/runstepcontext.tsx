import {createContext, useContext, useState, ReactNode} from 'react';
import {Template} from '../utils/types.js';

type RunStepContextType = {
	step: string;
	setStep: (step: string) => void;
	template: Template | null;
	setTemplate: (template: Template) => void;
	directory: string | null;
	setDirectory: (directory: string | null) => void;
	sessionId: string | null;
	setSessionId: (id: string | null) => void;
};

const RunStepContext = createContext<RunStepContextType | undefined>(undefined);

export const RunStepProvider = ({children}: {children: ReactNode}) => {
	// For run command, start at template step instead of projectname
	const [step, setStep] = useState<string>('template');
	const [template, setTemplate] = useState<Template | null>(null);
	const [directory, setDirectory] = useState<string | null>(null);
	const [sessionId, setSessionId] = useState<string | null>(null);

	return (
		<RunStepContext.Provider
			value={{
				step,
				setStep,
				template,
				setTemplate,
				directory,
				setDirectory,
				sessionId,
				setSessionId,
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
