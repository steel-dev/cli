import React, {createContext, useContext, useState, ReactNode} from 'react';
import {Template} from '../utils/types.js';

type StepContextType = {
	step: string;
	setStep: (step: string) => void;
	template: Template | null;
	setTemplate: (template: Template) => void;
	directory: string;
	setDirectory: (dir: string) => void;
	packageManager: string;
	setPackageManager: (manager: string) => void;
};

const StepContext = createContext<StepContextType | undefined>(undefined);

export const StepProvider = ({children}: {children: ReactNode}) => {
	const [step, setStep] = useState<string>('projectname');
	const [template, setTemplate] = useState<Template | null>(null);
	const [directory, setDirectory] = useState<string>('steel-project');
	const [packageManager, setPackageManager] = useState<string>('npm');
	return (
		<StepContext.Provider
			value={{
				step,
				setStep,
				template,
				setTemplate,
				directory,
				setDirectory,
				packageManager,
				setPackageManager,
			}}
		>
			{children}
		</StepContext.Provider>
	);
};

export const useStep = () => {
	const context = useContext(StepContext);
	if (!context) throw new Error('useStep must be used within a StepProvider');
	return context;
};
