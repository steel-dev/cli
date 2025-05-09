// import fs from 'node:fs';
// import path from 'node:path';
import React, {useState} from 'react';
import SteelApiKey from '../components/cookbook/steelapikey.js';
import Template from '../components/cookbook/template.js';
// import {fileURLToPath} from 'node:url';
// import SelectInput from 'ink-select-input';
import Directory from '../components/cookbook/directory.js';
import {TaskList} from 'ink-task-list';
import ProjectScaffolding from '../components/cookbook/projectscaffolding.js';
import Dependencies from '../components/cookbook/dependencies.js';
import zod from 'zod';

export const args = zod.tuple([
	zod
		.string()
		.default('steel-project')
		.describe('Directory to scaffold new Steel project'),
]);

type Props = {
	args: zod.infer<typeof args>;
};

export default function Cookbook({args}: Props) {
	// const renameFiles: Record<string, string | undefined> = {
	// 	_gitignore: '.gitignore',
	// };

	// const targetDir = 'steel-project';
	const [step, setStep] = useState<string>('template');
	// const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(
	// 	null,
	// );

	// const [directoryAction, setDirectoryAction] = useState<string | null>(null);

	function createDirectory() {
		const root = path.join(cwd, targetDir);
		fs.mkdirSync(root, {recursive: true});

		const pkgManager = pkgInfo ? pkgInfo.name : 'npm';

		console.log(`Scaffolding project in ${root}...`);

		const templateDir = path.resolve(
			fileURLToPath(import.meta.url),
			'../../examples',
			selectedTemplate?.value,
		);

		const write = (file: string, content?: string) => {
			const targetPath = path.join(root, renameFiles[file] ?? file);
			if (content) {
				fs.writeFileSync(targetPath, content);
			} else {
				copy(path.join(templateDir, file), targetPath);
			}
		};

		const files = fs.readdirSync(templateDir);
		for (const file of files.filter(f => f !== 'package.json')) {
			write(file);
		}

		// Handle package.json if it exists (for JS/TS projects)
		const packageJsonPath = path.join(templateDir, `package.json`);
		if (fs.existsSync(packageJsonPath)) {
			const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
			pkg.name = packageName;
			write('package.json', JSON.stringify(pkg, null, 2) + '\n');
		}
	}

	// // 3. Get package name
	// let packageName = path.basename(path.resolve(targetDir));
	// if (!isValidPackageName(packageName)) {
	// 	const packageNameResult = await prompts.text({
	// 		message: 'Package name:',
	// 		defaultValue: toValidPackageName(packageName),
	// 		placeholder: toValidPackageName(packageName),
	// 		validate(dir) {
	// 			if (!isValidPackageName(dir)) {
	// 				return 'Invalid package.json name';
	// 			}
	// 		},
	// 	});
	// 	if (prompts.isCancel(packageNameResult)) return cancel();
	// 	packageName = packageNameResult;
	// }

	// // 4. Choose a template
	// let templateName = argTemplate;
	// let template = TEMPLATES.find(t => t.name === templateName);
	// let hasInvalidArgTemplate = false;
	// if (argTemplate && !TEMPLATE_NAMES.includes(argTemplate)) {
	// 	templateName = undefined;
	// 	hasInvalidArgTemplate = true;
	// }

	// if (!templateName) {
	// 	const selectedTemplate = await prompts.select({
	// 		message: hasInvalidArgTemplate
	// 			? `"${argTemplate}" isn't a valid template. Please choose from below: `
	// 			: 'Select a starting template:',
	// 		options: TEMPLATES.map(template => {
	// 			const templateColor = template.color;
	// 			return {
	// 				label: templateColor(template.display || template.name),
	// 				value: template,
	// 			};
	// 		}),
	// 	});
	// 	if (prompts.isCancel(selectedTemplate)) return cancel();
	// 	template = selectedTemplate;
	// 	templateName = selectedTemplate.name;
	// } else {
	// 	// Find the framework from the template name
	// 	template = TEMPLATES.find(t => t.name === templateName);
	// 	if (!template) {
	// 		template = TEMPLATES[0];
	// 		templateName = TEMPLATES[0].name;
	// 	}
	// }

	// const root = path.join(cwd, targetDir);
	// fs.mkdirSync(root, {recursive: true});

	// const pkgManager = pkgInfo ? pkgInfo.name : 'npm';

	// prompts.log.step(`Scaffolding project in ${root}...`);

	// const templateDir = path.resolve(
	// 	fileURLToPath(import.meta.url),
	// 	'../../examples',
	// 	templateName,
	// );

	const write = (file: string, content?: string) => {
		const targetPath = path.join(root, renameFiles[file] ?? file);
		if (content) {
			fs.writeFileSync(targetPath, content);
		} else {
			copy(path.join(templateDir, file), targetPath);
		}
	};

	// const files = fs.readdirSync(templateDir);
	// for (const file of files.filter(f => f !== 'package.json')) {
	// 	write(file);
	// }

	// // Handle package.json if it exists (for JS/TS projects)
	// const packageJsonPath = path.join(templateDir, `package.json`);
	// if (fs.existsSync(packageJsonPath)) {
	// 	const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
	// 	pkg.name = packageName;
	// 	write('package.json', JSON.stringify(pkg, null, 2) + '\n');
	// }

	// // Ask for Steel API key
	// const steelApiKey = await prompts.text({
	// 	message: `Enter your ${yellow('Steel')} API key (press Enter to skip):`,
	// 	placeholder: 'ste-...',
	// });

	// // Copy .env.example to .env
	// if (fs.existsSync(path.join(root, '.env.example'))) {
	// 	fs.copyFileSync(path.join(root, '.env.example'), path.join(root, '.env'));

	// 	if (!prompts.isCancel(steelApiKey) && steelApiKey) {
	// 		// Replace STEEL_API_KEY in the .env file
	// 		const envPath = path.join(root, '.env');
	// 		let envContent = fs.readFileSync(envPath, 'utf-8');
	// 		envContent = envContent.replace(
	// 			/STEEL_API_KEY=.*/,
	// 			`STEEL_API_KEY=${steelApiKey}`,
	// 		);
	// 		fs.writeFileSync(envPath, envContent);
	// 	}
	// }

	// // Ask if user wants to install dependencies only for JS/TS projects (not Python)
	// let shouldInstall: boolean | symbol = false;
	// if (
	// 	!template.customCommands &&
	// 	fs.existsSync(path.join(root, 'package.json'))
	// ) {
	// 	shouldInstall = await prompts.confirm({
	// 		message: 'Do you want to install dependencies now?',
	// 		initialValue: true,
	// 	});

	// 	if (prompts.isCancel(shouldInstall)) {
	// 		return cancel();
	// 	}
	// }

	// if (shouldInstall) {
	// 	// cd to the project
	// 	process.chdir(root);
	// 	prompts.log.step('Installing dependencies...');
	// 	try {
	// 		// Run npm install or yarn
	// 		execSync(`${pkgManager} install`, {stdio: 'inherit'});
	// 		prompts.log.success('Dependencies installed successfully!');
	// 	} catch (error) {
	// 		prompts.log.error('Failed to install dependencies.');
	// 		console.error(error);
	// 	}
	// }

	// let doneMessage = '';
	// const cdProjectName = path.relative(cwd, root);
	// doneMessage += `Done. Now run:\n`;
	// if (root !== cwd) {
	// 	doneMessage += `\n  cd ${
	// 		cdProjectName.includes(' ') ? `"${cdProjectName}"` : cdProjectName
	// 	}`;
	// }
	// if (template.customCommands) {
	// 	for (const command of template.customCommands) {
	// 		doneMessage += `\n  ${command}`;
	// 	}
	// } else {
	// 	switch (pkgManager) {
	// 		case 'yarn':
	// 			doneMessage += '\n  yarn';
	// 			doneMessage += '\n  yarn start';
	// 			break;
	// 		default:
	// 			if (!shouldInstall) {
	// 				doneMessage += `\n  ${pkgManager} install`;
	// 			}
	// 			doneMessage += `\n  ${pkgManager} start`;
	// 			break;
	// 	}
	// }

	// // Only show API key instructions if they didn't provide one
	// const hasProvidedApiKey = !prompts.isCancel(steelApiKey) && !!steelApiKey;
	// const envVarsToAdd = hasProvidedApiKey
	// 	? template.extraEnvVarsRequired || []
	// 	: [
	// 			{name: 'STEEL_API_KEY', display: 'Steel API key'},
	// 			...(template.extraEnvVarsRequired || []),
	// 		];

	// // prettier-ignore
	// doneMessage +=`

	//  ${envVarsToAdd.length ? `${yellow("Important:")} Add your ${envVarsToAdd.map(e => e.display).join(" + ")} to the .env file
	//  Get a free API key at: ${blueBright("https://app.steel.dev/settings/api-keys")}
	//  ` : ''}
	//  Learn more about Steel at: ${blueBright("https://docs.steel.dev/")}`;

	// prompts.outro(doneMessage);
	return (
		<TaskList>
			<Template step={step} setStep={setStep} />
			<Directory step={step} setStep={setStep} args={args} />
			<ProjectScaffolding step={step} setStep={setStep} />
			<SteelApiKey step={step} setStep={setStep} />
			<Dependencies step={step} setStep={setStep} />
		</TaskList>
	);
}
