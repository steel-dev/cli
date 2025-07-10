import {createContext, useContext, useState, ReactNode} from 'react';
import {Template} from '../utils/types.js';

type ForgeStepContextType = {
	step: string;
	setStep: (step: string) => void;
	template: Template | null;
	setTemplate: (template: Template) => void;
	directory: string;
	setDirectory: (dir: string) => void;
	packageManager: string;
	setPackageManager: (manager: string) => void;
};

const ForgeStepContext = createContext<ForgeStepContextType | undefined>(
	undefined,
);

export const ForgeStepProvider = ({children}: {children: ReactNode}) => {
	const [step, setStep] = useState<string>('projectname');
	const [template, setTemplate] = useState<Template | null>(null);
	const [directory, setDirectory] = useState<string>('steel-project');
	const [packageManager, setPackageManager] = useState<string>('npm');
	return (
		<ForgeStepContext.Provider
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
		</ForgeStepContext.Provider>
	);
};

export const useForgeStep = () => {
	const context = useContext(ForgeStepContext);
	if (!context)
		throw new Error('useForgeStep must be used within a ForgeStepProvider');
	return context;
};
