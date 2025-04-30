import {appendToTopofFile, replaceString, wrapStringinFile} from './files.js';

const requiredImportsPuppeteerJs = [
	/from ['"]puppeteer-core['"]/,
	/from ['"]puppeteer['"]/,
];
const codePatternsPuppeteerJs = [/puppeteer.launch()/];

const configPuppeteerJs = (file: string) => {
	appendToTopofFile(
		file,
		`import Steel from "steel-sdk";
import dotenv from "dotenv";

dotenv.config();

const STEEL_API_KEY = process.env.STEEL_API_KEY;
// Initialize Steel client with the API key from environment variables
const client = new Steel({
  steelAPIKey: STEEL_API_KEY,
});`,
	);
	replaceString(
		file,
		'await puppeteer.launch()',
		` console.log("Creating Steel session...");

    // Create a new Steel session with all available options
    session = await client.sessions.create({
      // === Basic Options ===
      // useProxy: true, // Use Steel's proxy network (residential IPs)
      // proxyUrl: 'http://...',         // Use your own proxy (format: protocol://username:password@host:port)
      // solveCaptcha: true,             // Enable automatic CAPTCHA solving
      // sessionTimeout: 1800000,        // Session timeout in ms (default: 5 mins)
      // === Browser Configuration ===
      // userAgent: 'custom-ua-string',  // Set a custom User-Agent
    });

    console.log(
      \`\x1b[1;93mSteel Session created!\x1b[0m\n` +
			`View session at \x1b[1;37m\${session.sessionViewerUrl}\x1b[0m\`
    );

    // Connect Puppeteer to the Steel session
    browser = await puppeteer.connect({
      browserWSEndpoint: \`wss://connect.steel.dev?apiKey=\${STEEL_API_KEY}&sessionId=\${session.id}\`,
    });

    console.log("Connected to browser via Puppeteer");`,
	);
	wrapStringinFile(
		file,
		'',
		'await browser.close()',
		` if (session) {
      console.log("Releasing session...");
      await client.sessions.release(session.id);
      console.log("Session released");
    }`,
	);
};

const requiredImportsPlaywrightJs = [/from ['"]playwright['"]/];
const codePatternsPlaywrightJs = [/launch()/];
const configPlaywrightJs = (file: string) => {
	appendToTopofFile(
		file,
		`import Steel from "steel-sdk";
import dotenv from "dotenv";

dotenv.config();

const STEEL_API_KEY = process.env.STEEL_API_KEY;
// Initialize Steel client with the API key from environment variables
const client = new Steel({
  steelAPIKey: STEEL_API_KEY,
});`,
	);
	replaceString(
		file,
		'await puppeteer.launch()',
		` console.log("Creating Steel session...");

    // Create a new Steel session with all available options
    session = await client.sessions.create({
      // === Basic Options ===
      // useProxy: true, // Use Steel's proxy network (residential IPs)
      // proxyUrl: 'http://...',         // Use your own proxy (format: protocol://username:password@host:port)
      // solveCaptcha: true,             // Enable automatic CAPTCHA solving
      // sessionTimeout: 1800000,        // Session timeout in ms (default: 5 mins)
      // === Browser Configuration ===
      // userAgent: 'custom-ua-string',  // Set a custom User-Agent
    });

    console.log(
      \`\x1b[1;93mSteel Session created!\x1b[0m\n` +
			`View session at \x1b[1;37m\${session.sessionViewerUrl}\x1b[0m\`
    );

    // Connect Puppeteer to the Steel session
    browser = await puppeteer.connect({
      browserWSEndpoint: \`wss://connect.steel.dev?apiKey=\${STEEL_API_KEY}&sessionId=\${session.id}\`,
    });

    console.log("Connected to browser via Puppeteer");`,
	);
	wrapStringinFile(
		file,
		'',
		'await browser.close()',
		` if (session) {
      console.log("Releasing session...");
      await client.sessions.release(session.id);
      console.log("Session released");
    }`,
	);
};

const requiredImportsPlaywrightPy = [/from playwright/];
const codePatternsPlaywrightPy = [/launch()/, /playwright/];
const configPlaywrightPy = (file: string) => {
	appendToTopofFile(
		file,
		`from steel import Steel
from dotenv import load_dotenv

load_dotenv()
STEEL_API_KEY = os.getenv('STEEL_API_KEY')

# Initialize Steel client with the API key from environment variables
client = Steel(
    steel_api_key=STEEL_API_KEY,
)

print("Creating Steel session...")

# Create a new Steel session with all available options
session = client.sessions.create(
    # === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
)

// Connect Puppeteer to the Steel session
`,
	);
	wrapStringinFile(
		file,
		'',
		'browser.close()',
		` if session:
      print("Releasing session...");
      client.sessions.release(session.id);
      print("Session released");
    `,
	);
};

const requiredImportsBrowserUsePy = [/from browser_use import Agent/];
const codePatternsBrowserUsePy = [/Agent/];
const configBrowserUsePy = (file: string) => {
	appendToTopofFile(
		file,
		`
		from steel import Steel
from dotenv import load_dotenv

load_dotenv()
STEEL_API_KEY = os.getenv('STEEL_API_KEY')

# Initialize Steel client with the API key from environment variables
client = Steel(
    steel_api_key=STEEL_API_KEY,
)

print("Creating Steel session...")

# Create a new Steel session with all available options
session = client.sessions.create(
    # === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
)
cdp_url = f"wss://connect.steel.dev?apiKey={STEEL_API_KEY}&sessionId={session.id}"
browser = Browser(config=BrowserConfig(cdp_url=cdp_url))
browser_context = BrowserContext(browser=browser)`,
	);
	wrapStringinFile(
		file,
		'',
		'llm=model',
		`browser=browser,
		browser_context=browser_context,`,
	);
};

const requiredImportsSeleniumPy = [/from selenium/];
const codePatternsSeleniumPy = [/webdriver/];
const configSeleniumPy = (file: string) => {
	appendToTopofFile(
		file,
		`
		from steel import Steel
from dotenv import load_dotenv
from selenium.webdriver.remote.remote_connection import RemoteConnection

load_dotenv()
STEEL_API_KEY = os.getenv('STEEL_API_KEY')

# Initialize Steel client with the API key from environment variables
client = Steel(
    steel_api_key=STEEL_API_KEY,
)

class CustomRemoteConnection(RemoteConnection):
    _session_id = None

    def __init__(self, remote_server_addr: str, session_id: str):
        super().__init__(remote_server_addr)
        self._session_id = session_id

    def get_remote_connection_headers(self, parsed_url, keep_alive=False):
        headers = super().get_remote_connection_headers(parsed_url, keep_alive)
        headers.update({'steel-api-key': os.environ.get("STEEL_API_KEY")})
        headers.update({'session-id': self._session_id})
        return headers


print("Creating Steel session...")

# Create a new Steel session with all available options
session = client.sessions.create(
    # === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
    is_selenium=True,
)

# Connect to the session via Selenium's WebDriver using the CustomRemoteConnection class
        driver = webdriver.Remote(
            command_executor=CustomRemoteConnection(
                remote_server_addr='http://connect.steelbrowser.com/selenium',
                session_id=session.id
            ),
            options=webdriver.ChromeOptions()
        )
`,
	);
};

export const possibleJsImports = [
	{
		name: 'puppeteer',
		imports: requiredImportsPuppeteerJs,
		codePatterns: codePatternsPuppeteerJs,
		config: configPuppeteerJs,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightJs,
		codePatterns: codePatternsPlaywrightJs,
		config: configPlaywrightJs,
	},
];
export const possiblePyImports = [
	{
		name: 'browser-use',
		imports: requiredImportsBrowserUsePy,
		codePatterns: codePatternsBrowserUsePy,
		config: configBrowserUsePy,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightPy,
		codePatterns: codePatternsPlaywrightPy,
		config: configPlaywrightPy,
	},
	{
		name: 'selenium',
		imports: requiredImportsSeleniumPy,
		codePatterns: codePatternsSeleniumPy,
		config: configSeleniumPy,
	},
];
