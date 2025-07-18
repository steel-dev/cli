import React, {createContext, useContext, useState, ReactNode} from 'react';
import Callout, {Variant} from '../components/callout.js';
import {v4 as uuid} from 'uuid';

type Message = {id: string; variant: Variant; title: string; body?: string};

const CalloutCtx = createContext({
	add: (_: Omit<Message, 'id'>) => {},
	remove: (_: string) => {},
	clear: () => {},
});

export function useCallout() {
	return useContext(CalloutCtx);
}

export function CalloutProvider({children}: {children: ReactNode}) {
	const [messages, setMessages] = useState<Message[]>([]);

	return (
		<CalloutCtx.Provider
			value={{
				add: message =>
					setMessages(current => [...current, {...message, id: uuid()}]),
				remove: id =>
					setMessages(current => current.filter(msg => msg.id !== id)),
				clear: () => setMessages([]),
			}}
		>
			{messages.map(message => (
				<Callout
					key={message.id}
					variant={message.variant}
					title={message.title}
				>
					{message.body}
				</Callout>
			))}
			{children}
		</CalloutCtx.Provider>
	);
}
