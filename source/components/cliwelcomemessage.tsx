import React from 'react';
import {Box, Text} from 'ink';
import chalk from 'chalk';

// prettier-ignore
const steelWelcomeMessage = `

${chalk.yellowBright(" @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ ")}
${chalk.yellowBright("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")}
${chalk.yellowBright("@@@@@@@@@9999999999999@@@@@@@@@")}      ${chalk.cyan("Humans use Chrome, Agents use Steel.")}
${chalk.yellowBright("@@@@@[                   ]@@@@@")}
${chalk.yellowBright("@@@@[                      @@@@")}
${chalk.yellowBright("@@@@     @@@@@@@@@@@@@     @@@@")}
${chalk.yellowBright("@@@@B              @@@     @@@@")}      Steel is an open-source browser API purpose-built for AI agents.
${chalk.yellowBright("@@@@@@               @     @@@@")}      Give one or 1,000 agents the ability to interact with any website.
${chalk.yellowBright("@@@@@@@@@@@@@@@@     @     @@@@")}
${chalk.yellowBright("@@@@                 @     @@@@")}
${chalk.yellowBright("@@@@                @@     @@@@")}
${chalk.yellowBright("@@@@@@@@@@@@@@@@g@@@@@@@@@@@@@@")}      ${chalk.green("Documentation:")} ${chalk.blueBright("https://docs.steel.dev/")}
${chalk.yellowBright("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")}      ${chalk.green("GitHub:")} ${chalk.blueBright("https://github.com/steel-dev/steel-browser")}
${chalk.yellowBright(" @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ ")}

`;

export default function CLIWelcomeMessage() {
	return (
		<Box flexDirection="column" marginBottom={2}>
			<Text>{steelWelcomeMessage}</Text>
		</Box>
	);
}
