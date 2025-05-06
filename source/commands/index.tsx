import React from 'react';
import {Text, Box} from 'ink';
import Link from 'ink-link';

export default function Index() {
	return (
		<Box flexDirection="column">
			<Text color="yellow">{` @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ `}</Text>
			<Text color="yellow">{`@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@`}</Text>
			<Text color="yellow">{`@@@@@@@@@9999999999999@@@@@@@@@`}</Text>
			<Text color="yellow">{`@@@@@[                   ]@@@@@`}</Text>
			<Text color="yellow">{`@@@@[                      @@@@`}</Text>
			<Text color="yellow">{`@@@@     @@@@@@@@@@@@@     @@@@`}</Text>
			<Text color="yellow">{`@@@@B              @@@     @@@@`}</Text>
			<Text color="yellow">{`@@@@@@               @     @@@@`}</Text>
			<Text color="yellow">{`@@@@@@@@@@@@@@@@     @     @@@@`}</Text>
			<Text color="yellow">{`@@@@                 @     @@@@`}</Text>
			<Text color="yellow">{`@@@@                @@     @@@@`}</Text>
			<Text color="yellow">{`@@@@@@@@@@@@@@@g@@@@@@@@@@@@@@@`}</Text>
			<Text color="yellow">{`@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@`}</Text>
			<Text color="yellow">{` @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ `}</Text>
			<Text color="cyan">Humans use Chrome, Agents use Steel.</Text>
			<Text>
				Steel is an open-source browser API purpose-built for AI agents.
			</Text>
			<Text>
				Give one or 1,000 agents the ability to interact with any website.
			</Text>
			<Link url="https://docs.steel.dev/">
				<Text color="green">Documentation</Text>
			</Link>
			<Link url="https://github.com/steel-dev/steel-browser">
				<Text color="green">GitHub</Text>
			</Link>
		</Box>
	);
}

// `
// 	Usage
// 	  $ steel-cli <command>

// 	Commands
// 	  init        Initialize a new project
// 	  start       Start the development server
// 		login       Login to the Steel Cloud
// 		logout      Logout from the Steel Cloud

// 		API Commands:
// 			sessions  Make requests to the Sessions API
// 			tools     Make requests to the Browser Tools API
// 			files     Make requests to the Files API

// 		cookbook    Sample recipes for using Steel
// 		integrate   Integrate Steel with your project
// 		version     Display the version number
// 		help        Display this help message
// `,
