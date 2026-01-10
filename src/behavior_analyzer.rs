/// Behavioral Analysis - Understands what code DOES, not what functions it uses
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct BehaviorProfile {
    pub purpose: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub category: CodeCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeCategory {
    // React-specific (11,184 createElement found!)
    ReactComponent,
    ReactHooks,
    ReactEventHandler,
    ReactStateManager,
    ReactContextProvider,

    // DOM Manipulation (11,184 createElement, 787 existsSync)
    DOMManipulator,
    DOMEventListener,
    VirtualDOMRenderer,

    // Process & System (3,026 process references)
    ProcessManager,
    ChildProcessSpawner,
    StdioHandler,
    EnvironmentVariableLoader,

    // Error Handling (8,632 throw, 8,520 Error())
    ErrorHandler,
    ErrorRecovery,
    ExceptionLogger,
    PromiseRejectionHandler,

    // Async/Promises (6,749 await, 3,907 async)
    AsyncOrchestrator,
    PromiseChain,
    AsyncIterator,

    // File System (787 existsSync, 278 statSync)
    FileSystemSync,
    FileSystemAsync,
    PathResolver,
    DirectoryWatcher,

    // String Operations (2,978 .join, 2,498 .slice)
    StringManipulator,
    RegexMatcher,
    TextParser,

    // Data Parsing (775 parseInt, 442 JSON.stringify)
    JSONParser,
    DataSerializer,
    URLEncoder,

    // Timing (1,000 Date.now, 660 setTimeout)
    TimerScheduler,
    PerformanceMonitor,
    AnimationFrameHandler,

    // Authentication (AWS, OAuth, JWT)
    AWSCredentialProvider,
    OAuthFlowController,
    JWTTokenValidator,
    SessionCookieManager,

    // State Management
    StateManagement,
    ReduxStore,
    MobXStore,
    ZustandStore,

    // Event Handling (280 event handlers found!)
    EventHandling,
    MessageQueueConsumer,
    WebSocketEventHandler,
    FileSystemEventWatcher,

    // Validation
    DataValidation,
    SchemaValidator,
    FormValidator,

    // Workflow
    WorkflowOrchestration,
    PipelineExecutor,
    TaskQueueProcessor,

    // Communication
    ApiClient,
    ProtocolHandler,
    MessageRouter,
    HTTPClient,
    WebSocketClient,

    // System
    PermissionControl,
    ResourceManagement,
    ConfigurationLoader,

    // Infrastructure
    TelemetryRecorder,
    LoggingSystem,
    CacheManager,

    // Claude-specific (Claude-Escapes, tengu_, CLAUDE_CODE_)
    ClaudeProtocolHandler,
    ClaudeTelemetryRecorder,
    ClaudeEnvironmentLoader,

    // Claude Code Internals (81 CLAUDE_CODE_ vars found!)
    SandboxManager,
    APIKeyVault,
    IDEConnector,
    CommandInjectionGuard,
    ProxyManager,
    TelemetryController,
    TokenBudgetManager,
    RetryPolicy,
    ToolConcurrencyLimiter,
    PromptSuggestionEngine,
    SDKCheckpointing,

    // Telemetry Specific (486 unique tengu_ events found!)
    AgentTelemetryRecorder,
    APIMonitoringDashboard,
    FeedbackCollectionSystem,
    SearchAnalytics,
    BashSecurityMonitor,
    GitHubIntegrationTracker,
    PlanModeAnalytics,
    MCPOperationTracker,
    ToolUseMonitor,
    VersionLockTracker,
    OAuthFlowTracker,
    TreeSitterLoader,

    // UI Systems
    KeyboardShortcutManager,
    CommandPaletteHandler,
    ModalDialogController,

    // Error Tracking
    SentryIntegration,

    // Specialized
    SyntaxHighlighter,
    LanguageParser,

    // React Component Library (from iteration 5)
    InputComponentLibrary,        // Input (1153)
    SelectComponentLibrary,       // Select (1128)
    FormComponentLibrary,         // Form (953)
    TabNavigationSystem,          // Tab (737)
    ProgressIndicatorSystem,      // Progress (424)
    AlertNotificationSystem,      // Alert (220)
    ButtonComponentLibrary,       // Button (211)
    DialogComponentLibrary,       // Dialog (200)
    MenuComponentLibrary,         // Menu (136)

    // State Management (from iteration 5)
    ActionDispatcher,             // action (1810), dispatch (179)
    StateSelector,                // selector (168)
    StoreManagerCore,             // store (410)

    // Network/API Layer (from iteration 5)
    HTTPRequestManager,           // http (4176), request (2449)
    HTTPResponseHandler,          // response (1451)
    EndpointRegistry,             // endpoint (1055)
    APIClientLibrary,             // api (1080)
    FetchAPIWrapper,              // fetch (709)

    // Editor Features (from iteration 5)
    DiffViewerComponent,          // diff (601)
    MergeConflictResolver,        // merge (635), conflict (62)
    CompactOperationManager,      // compact (353)
    TeleportNavigator,            // teleport (112)

    // Service Architecture (from iteration 5)
    PlaneServiceCoordinator,      // PlaneService (188)
    CredentialsProviderSystem,    // CredentialsProvider (112)

    // Error Types (from iteration 6)
    TypeErrorHandler,             // TypeError (1457)
    ParameterErrorHandler,        // ParameterError (350)
    ProviderErrorHandler,         // ProviderError (156)
    RangeErrorHandler,            // RangeError (144)
    ServiceExceptionHandler,      // ServiceException (112)

    // Additional Error Types (iteration 8)
    AbortErrorHandler,            // AbortError (88)
    SyntaxErrorHandler,           // SyntaxError (82)
    TimeoutErrorHandler,          // TimeoutError (76)
    UnknownErrorHandler,          // UnknownError (62)
    QueryErrorHandler,            // QueryError (62)
    ReferenceErrorHandler,        // ReferenceError (59)
    ParseErrorHandler,            // ParseError (58)
    ResponseErrorHandler,         // ResponseError (54)
    RequestErrorHandler,          // RequestError (54)
    InternalErrorHandler,         // InternalError (54)
    TokenExceptionHandler,        // TokenException (48)
    ServerExceptionHandler,       // ServerException (44)
    AxiosErrorHandler,            // AxiosError (44)
    ParserErrorHandler,           // ParserError (42)
    AuthErrorHandler,             // AuthError (42)

    // Content-Based Categories (Iteration 13 - defeats filename obfuscation!)
    EllipticCurveCrypto,          // EC, KeyPair, Signature, HmacDRBG
    NodeErrorFactory,             // createErrorType, NodeError, codes[]
    BitwiseCryptoOps,             // ushrn, bitLength, nh (crypto bit operations)
    JavaScriptSyntaxHighlighter,  // JSX tags, keyword highlighting, className
    ReactDevToolsProfiler,        // __REACT_DEVTOOLS_GLOBAL_HOOK__, performance profiling
    SyncFileIO,                   // sync file operations
    ImageProcessor,               // image handling and manipulation
    RegexEngine,                  // regex compilation and execution
    TimestampManager,             // timestamp creation and formatting
    APIErrorHandler,              // API-specific error handling
    ErrorRecoverySystem,          // error recovery strategies
    FallbackErrorHandler,         // fallback error handling
    PromiseErrorHandler,          // promise rejection handling

    // Iteration 14: Additional obfuscated utilities discovered!
    DateFnsLibrary,               // weekStartsOn, Date.UTC, getFullYear, locale-aware dates
    DebounceThrottle,             // leading, trailing, maxWait, setTimeout/clearTimeout
    JSONTokenizer,                // brace matching, token types, JSON.parse
    StartupProfiler,              // performance marks, startup timing, profiling reports
    ObjectInspector,              // property introspection, __proto__, constructor.name
    ElmSyntaxHighlighter,         // Elm language: infix/infixl/infixr, port keyword
    LodashTypeChecker,            // [object Array], [object Function], type introspection

    // Iteration 20: Deep obfuscation - entire directories with misleading names!
    RxJSOperators,                // .subscribe(), createOperatorSubscriber, Observable, Subject
    OpenTelemetryEncoding,        // hrTimeToNanos, hexToBinary, encodeAsLongBits, createInstrumentationScope
    ZlibCompression,              // Z_MIN_WINDOWBITS, Z_DEFAULT_CHUNK, deflate, inflate, gzip
    InstallationDetection,        // Homebrew, winget, process.execPath, npm config get prefix
    AnthropicAPIClient,           // api.anthropic.com, CLAUDE_CODE_USE_BEDROCK, session logs
    LodashCoreLibrary,            // Heavy typeof checks, Object.keys, type coercion, minified patterns

    // Iteration 24: Crypto library wrappers and entry points
    CryptoLibraryWrappers,        // browserify-crypto, ASN.1, elliptic lib, hash lib entry points

    Unknown,
}

pub struct BehaviorAnalyzer {
    code: String,
}

impl BehaviorAnalyzer {
    pub fn new(code: String) -> Self {
        Self { code }
    }

    /// Main analysis: What does this code actually DO?
    pub fn analyze(&self) -> BehaviorProfile {
        let mut evidence = Vec::new();
        let mut scores: HashMap<CodeCategory, f64> = HashMap::new();

        // Analyze different behavioral aspects
        // Check for syntax highlighters/language parsers FIRST (high priority)
        self.analyze_syntax_highlighter(&mut scores, &mut evidence);

        // ITERATION 13: Content-based detection to defeat filename obfuscation
        self.analyze_content_based_categories(&mut scores, &mut evidence);

        // React patterns (11,184 createElement!)
        self.analyze_react(&mut scores, &mut evidence);

        // Process patterns (3,026 process refs)
        self.analyze_process(&mut scores, &mut evidence);

        // File system patterns (787 existsSync)
        self.analyze_filesystem(&mut scores, &mut evidence);

        // String operations (2,978 .join, 2,498 .slice)
        self.analyze_string_ops(&mut scores, &mut evidence);

        // Data parsing (775 parseInt, 442 JSON.stringify)
        self.analyze_data_parsing(&mut scores, &mut evidence);

        // Timing (1,000 Date.now, 660 setTimeout)
        self.analyze_timing(&mut scores, &mut evidence);

        // Claude-specific patterns
        self.analyze_claude_specific(&mut scores, &mut evidence);

        // Iteration 5-6 discoveries: React components, network, state, editor, services, errors
        self.analyze_react_components(&mut scores, &mut evidence);
        self.analyze_network_layer(&mut scores, &mut evidence);
        self.analyze_state_patterns(&mut scores, &mut evidence);
        self.analyze_editor_features(&mut scores, &mut evidence);
        self.analyze_services(&mut scores, &mut evidence);
        self.analyze_error_types(&mut scores, &mut evidence);

        // Original analyzers
        self.analyze_state_machine(&mut scores, &mut evidence);
        self.analyze_validation(&mut scores, &mut evidence);
        self.analyze_authentication(&mut scores, &mut evidence);
        self.analyze_error_handling(&mut scores, &mut evidence);
        self.analyze_telemetry(&mut scores, &mut evidence);
        self.analyze_permissions(&mut scores, &mut evidence);
        self.analyze_protocol(&mut scores, &mut evidence);

        // DEBUG: Print all scores if bash security is detected
        if self.code.contains("tengu_bash_security_check_triggered") {
            eprintln!("\n=== BASH SECURITY CODE DETECTED ===");
            let mut sorted_scores: Vec<_> = scores.iter().collect();
            sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            for (cat, score) in sorted_scores.iter().take(10) {
                eprintln!("{:?}: {:.2}", cat, score);
            }
            eprintln!("Evidence: {:?}\n", evidence);
        }

        // Find highest scoring category
        let (category, confidence) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or((CodeCategory::Unknown, 0.0));

        let purpose = self.infer_purpose(&category, &evidence);

        BehaviorProfile {
            purpose,
            confidence,
            evidence,
            category,
        }
    }

    /// Detect React patterns
    fn analyze_react(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let create_element_count = self.code.matches("createElement").count();
        if create_element_count > 5 {
            *scores.entry(CodeCategory::ReactComponent).or_insert(0.0) += (create_element_count as f64 * 0.1).min(3.0);
            evidence.push(format!("React ({} createElement)", create_element_count));
        }
    }

    fn analyze_process(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let process_count = self.code.matches("process.").count();
        if process_count > 5 {
            *scores.entry(CodeCategory::ProcessManager).or_insert(0.0) += (process_count as f64 * 0.1).min(2.0);
            evidence.push("Process management".to_string());
        }
    }

    fn analyze_filesystem(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let fs_count = self.code.matches("Sync(").count();
        if fs_count > 3 {
            *scores.entry(CodeCategory::FileSystemSync).or_insert(0.0) += (fs_count as f64 * 0.2).min(2.0);
            evidence.push("File system operations".to_string());
        }
    }

    fn analyze_string_ops(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let str_ops = self.code.matches(".join(").count() + self.code.matches(".slice(").count();
        if str_ops > 10 {
            *scores.entry(CodeCategory::StringManipulator).or_insert(0.0) += (str_ops as f64 * 0.05).min(2.0);
            evidence.push("String operations".to_string());
        }
    }

    fn analyze_data_parsing(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let json_count = self.code.matches("JSON.").count();
        if json_count > 2 {
            *scores.entry(CodeCategory::JSONParser).or_insert(0.0) += (json_count as f64 * 0.3).min(2.0);
            evidence.push("JSON parsing".to_string());
        }
    }

    fn analyze_timing(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let timer_count = self.code.matches("setTimeout").count() + self.code.matches("setInterval").count();
        if timer_count > 0 {
            *scores.entry(CodeCategory::TimerScheduler).or_insert(0.0) += timer_count as f64 * 0.4;
            evidence.push("Timer scheduling".to_string());
        }
    }

    fn analyze_claude_specific(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Claude Protocol Headers
        if self.code.contains("Claude-Escapes") || self.code.contains("Claude-Steers") {
            *scores.entry(CodeCategory::ClaudeProtocolHandler).or_insert(0.0) += 2.5;
            evidence.push("Claude protocol headers".to_string());
        }

        // Sandbox Management (highest specificity)
        if self.code.contains("CLAUDE_CODE_BUBBLEWRAP") || self.code.contains("BASH_SANDBOX_SHOW_INDICATOR") {
            *scores.entry(CodeCategory::SandboxManager).or_insert(0.0) += 10.0;
            evidence.push("Sandbox management (BUBBLEWRAP)".to_string());
        }

        // API Key Vault
        if self.code.contains("API_KEY_FILE_DESCRIPTOR") ||
           (self.code.contains("API_KEY_HELPER") && self.code.contains("TTL_MS")) {
            *scores.entry(CodeCategory::APIKeyVault).or_insert(0.0) += 10.0;
            evidence.push("API Key Vault".to_string());
        }

        // IDE Connector
        if self.code.contains("AUTO_CONNECT_IDE") || self.code.contains("VSCODE_SETTINGS") {
            *scores.entry(CodeCategory::IDEConnector).or_insert(0.0) += 2.5;
            evidence.push("IDE connector".to_string());
        }

        // Command Injection Guard
        if self.code.contains("DISABLE_COMMAND_INJECTION_CHECK") || self.code.contains("ADDITIONAL_PROTECTION") {
            *scores.entry(CodeCategory::CommandInjectionGuard).or_insert(0.0) += 2.5;
            evidence.push("Command injection guard".to_string());
        }

        // Agent Telemetry (specific tengu_ events)
        if self.code.contains("tengu_agent_") {
            *scores.entry(CodeCategory::AgentTelemetryRecorder).or_insert(0.0) += 2.5;
            evidence.push("Agent telemetry (tengu_agent_)".to_string());
        }

        // API Monitoring
        if self.code.contains("tengu_api_error") || self.code.contains("tengu_api_retry") {
            *scores.entry(CodeCategory::APIMonitoringDashboard).or_insert(0.0) += 2.5;
            evidence.push("API monitoring (tengu_api_)".to_string());
        }

        // Feedback Collection
        if self.code.contains("tengu_accept_feedback") || self.code.contains("tengu_accept_submitted") {
            *scores.entry(CodeCategory::FeedbackCollectionSystem).or_insert(0.0) += 2.5;
            evidence.push("Feedback collection".to_string());
        }

        // Search Analytics
        if self.code.contains("tengu_agentic_search") || self.code.contains("tengu_search_") {
            *scores.entry(CodeCategory::SearchAnalytics).or_insert(0.0) += 2.5;
            evidence.push("Search analytics".to_string());
        }

        // Keyboard Shortcuts (HIGHEST specificity - data structure pattern)
        if (self.code.contains("bindings:") || self.code.contains("bindings={")) &&
           (self.code.contains("ctrl+") || self.code.contains("meta+")) {
            *scores.entry(CodeCategory::KeyboardShortcutManager).or_insert(0.0) += 10.0;
            evidence.push("Keyboard shortcut bindings".to_string());
        }

        // Command Palette
        if (self.code.contains("\"app:") || self.code.contains("\"chat:")) &&
           self.code.contains("\"escape") {
            *scores.entry(CodeCategory::CommandPaletteHandler).or_insert(0.0) += 2.5;
            evidence.push("Command palette (app:*, chat:*)".to_string());
        }

        // Sentry Integration (HIGHEST specificity - unique string combo)
        if self.code.contains("SENTRY_DSN") && self.code.contains("tracesSampleRate") {
            *scores.entry(CodeCategory::SentryIntegration).or_insert(0.0) += 10.0;
            evidence.push("Sentry error tracking".to_string());
        }

        // Modal Dialog (escape + confirmation patterns)
        if self.code.contains("escape:") &&
           (self.code.contains("confirm:") || self.code.contains("Confirmation")) {
            *scores.entry(CodeCategory::ModalDialogController).or_insert(0.0) += 2.0;
            evidence.push("Modal dialog controller".to_string());
        }

        // Generic Claude telemetry (fallback)
        if self.code.contains("tengu_") {
            *scores.entry(CodeCategory::ClaudeTelemetryRecorder).or_insert(0.0) += 1.0;
            evidence.push("Claude telemetry (generic)".to_string());
        }

        // Proxy Manager
        if self.code.contains("HOST_HTTP_PROXY_PORT") || self.code.contains("HOST_SOCKS_PROXY_PORT") {
            *scores.entry(CodeCategory::ProxyManager).or_insert(0.0) += 10.0;
            evidence.push("Proxy manager (HTTP/SOCKS)".to_string());
        }

        // Telemetry Controller
        if self.code.contains("ENABLE_TELEMETRY") || self.code.contains("OTEL_FLUSH_TIMEOUT_MS") {
            *scores.entry(CodeCategory::TelemetryController).or_insert(0.0) += 2.5;
            evidence.push("Telemetry controller".to_string());
        }

        // Token Budget Manager
        if self.code.contains("FILE_READ_MAX_OUTPUT_TOKENS") || self.code.contains("MAX_OUTPUT_TOKENS") {
            *scores.entry(CodeCategory::TokenBudgetManager).or_insert(0.0) += 10.0;
            evidence.push("Token budget manager".to_string());
        }

        // Retry Policy
        if self.code.contains("MAX_RETRIES") && self.code.contains("CLAUDE_CODE") {
            *scores.entry(CodeCategory::RetryPolicy).or_insert(0.0) += 2.5;
            evidence.push("Retry policy".to_string());
        }

        // Tool Concurrency Limiter
        if self.code.contains("MAX_TOOL_USE_CONCURRENCY") {
            *scores.entry(CodeCategory::ToolConcurrencyLimiter).or_insert(0.0) += 10.0;
            evidence.push("Tool concurrency limiter".to_string());
        }

        // Prompt Suggestion Engine
        if self.code.contains("ENABLE_PROMPT_SUGGESTION") || self.code.contains("tengu_prompt_suggestion") {
            *scores.entry(CodeCategory::PromptSuggestionEngine).or_insert(0.0) += 2.5;
            evidence.push("Prompt suggestion engine".to_string());
        }

        // SDK Checkpointing
        if self.code.contains("ENABLE_SDK_FILE_CHECKPOINTING") {
            *scores.entry(CodeCategory::SDKCheckpointing).or_insert(0.0) += 2.5;
            evidence.push("SDK checkpointing".to_string());
        }

        // Bash Security Monitor (52 occurrences!) - HIGHEST SPECIFICITY
        if self.code.contains("tengu_bash_security_check_triggered") {
            *scores.entry(CodeCategory::BashSecurityMonitor).or_insert(0.0) += 10.0;
            evidence.push("Bash security monitor".to_string());
        }

        // GitHub Integration Tracker
        if self.code.contains("tengu_install_github_app") || self.code.contains("tengu_setup_github_actions") {
            *scores.entry(CodeCategory::GitHubIntegrationTracker).or_insert(0.0) += 2.5;
            evidence.push("GitHub integration tracker".to_string());
        }

        // Plan Mode Analytics
        if self.code.contains("tengu_plan_exit") || self.code.contains("tengu_plan_") {
            *scores.entry(CodeCategory::PlanModeAnalytics).or_insert(0.0) += 2.5;
            evidence.push("Plan mode analytics".to_string());
        }

        // MCP Operation Tracker
        if self.code.contains("tengu_mcp_") {
            *scores.entry(CodeCategory::MCPOperationTracker).or_insert(0.0) += 2.5;
            evidence.push("MCP operation tracker".to_string());
        }

        // Tool Use Monitor
        if self.code.contains("tengu_tool_use_error") || self.code.contains("tengu_tool_use_success") {
            *scores.entry(CodeCategory::ToolUseMonitor).or_insert(0.0) += 2.5;
            evidence.push("Tool use monitor".to_string());
        }

        // Version Lock Tracker
        if self.code.contains("tengu_version_lock_") {
            *scores.entry(CodeCategory::VersionLockTracker).or_insert(0.0) += 2.5;
            evidence.push("Version lock tracker".to_string());
        }

        // OAuth Flow Tracker
        if self.code.contains("tengu_oauth_") {
            *scores.entry(CodeCategory::OAuthFlowTracker).or_insert(0.0) += 2.5;
            evidence.push("OAuth flow tracker".to_string());
        }

        // Tree-sitter Loader
        if self.code.contains("tengu_tree_sitter_load") || self.code.contains("tree-sitter") {
            *scores.entry(CodeCategory::TreeSitterLoader).or_insert(0.0) += 2.5;
            evidence.push("Tree-sitter parser loader".to_string());
        }

        // Generic environment loader (fallback)
        let claude_env_count = self.code.matches("CLAUDE_CODE_").count();
        if claude_env_count > 3 {
            *scores.entry(CodeCategory::ClaudeEnvironmentLoader).or_insert(0.0) += (claude_env_count as f64 * 0.2).min(1.5);
            evidence.push(format!("Claude environment ({} vars)", claude_env_count));
        }
    }

    /// Analyze React UI components (Iteration 5 discoveries)
    fn analyze_react_components(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Input components (1153 occurrences!)
        let input_count = self.code.matches("Input").count();
        if input_count >= 5 {
            *scores.entry(CodeCategory::InputComponentLibrary).or_insert(0.0) += (input_count as f64 * 0.3).min(10.0);
            evidence.push(format!("Input component library ({} refs)", input_count));
        }

        // Select components (1128 occurrences!)
        let select_count = self.code.matches("Select").count();
        if select_count >= 5 {
            *scores.entry(CodeCategory::SelectComponentLibrary).or_insert(0.0) += (select_count as f64 * 0.3).min(10.0);
            evidence.push(format!("Select component library ({} refs)", select_count));
        }

        // Form components (953 occurrences!)
        let form_count = self.code.matches("Form").count();
        if form_count >= 5 {
            *scores.entry(CodeCategory::FormComponentLibrary).or_insert(0.0) += (form_count as f64 * 0.3).min(10.0);
            evidence.push(format!("Form component library ({} refs)", form_count));
        }

        // Tab navigation (737 occurrences!)
        let tab_count = self.code.matches("Tab").count();
        if tab_count >= 5 {
            *scores.entry(CodeCategory::TabNavigationSystem).or_insert(0.0) += (tab_count as f64 * 0.3).min(8.0);
            evidence.push(format!("Tab navigation ({} refs)", tab_count));
        }

        // Progress indicators (424 occurrences!)
        let progress_count = self.code.matches("Progress").count();
        if progress_count >= 5 {
            *scores.entry(CodeCategory::ProgressIndicatorSystem).or_insert(0.0) += (progress_count as f64 * 0.3).min(8.0);
            evidence.push(format!("Progress indicators ({} refs)", progress_count));
        }

        // Alert/notification system (220 occurrences!)
        let alert_count = self.code.matches("Alert").count();
        if alert_count >= 3 {
            *scores.entry(CodeCategory::AlertNotificationSystem).or_insert(0.0) += (alert_count as f64 * 0.4).min(8.0);
            evidence.push(format!("Alert system ({} refs)", alert_count));
        }

        // Button library (211 occurrences!)
        let button_count = self.code.matches("Button").count();
        if button_count >= 3 {
            *scores.entry(CodeCategory::ButtonComponentLibrary).or_insert(0.0) += (button_count as f64 * 0.4).min(8.0);
            evidence.push(format!("Button library ({} refs)", button_count));
        }

        // Dialog components (200 occurrences!)
        let dialog_count = self.code.matches("Dialog").count();
        if dialog_count >= 3 {
            *scores.entry(CodeCategory::DialogComponentLibrary).or_insert(0.0) += (dialog_count as f64 * 0.4).min(8.0);
            evidence.push(format!("Dialog components ({} refs)", dialog_count));
        }

        // Menu components (136 occurrences!)
        let menu_count = self.code.matches("Menu").count();
        if menu_count >= 3 {
            *scores.entry(CodeCategory::MenuComponentLibrary).or_insert(0.0) += (menu_count as f64 * 0.4).min(7.0);
            evidence.push(format!("Menu components ({} refs)", menu_count));
        }
    }

    /// Analyze network/API layer (Iteration 5 discoveries)
    fn analyze_network_layer(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // HTTP request management (4176 http, 2449 request!)
        let http_count = self.code.matches("http").count();
        let request_count = self.code.matches("request").count();
        if http_count >= 10 || request_count >= 10 {
            *scores.entry(CodeCategory::HTTPRequestManager).or_insert(0.0) += ((http_count + request_count) as f64 * 0.05).min(10.0);
            evidence.push(format!("HTTP request manager ({} http, {} request)", http_count, request_count));
        }

        // HTTP response handling (1451 response!)
        let response_count = self.code.matches("response").count();
        if response_count >= 10 {
            *scores.entry(CodeCategory::HTTPResponseHandler).or_insert(0.0) += (response_count as f64 * 0.1).min(10.0);
            evidence.push(format!("HTTP response handler ({} refs)", response_count));
        }

        // Endpoint registry (1055 endpoint!)
        let endpoint_count = self.code.matches("endpoint").count();
        if endpoint_count >= 5 {
            *scores.entry(CodeCategory::EndpointRegistry).or_insert(0.0) += (endpoint_count as f64 * 0.2).min(9.0);
            evidence.push(format!("Endpoint registry ({} refs)", endpoint_count));
        }

        // API client library (1080 api!)
        let api_count = self.code.matches("api").count();
        if api_count >= 10 {
            *scores.entry(CodeCategory::APIClientLibrary).or_insert(0.0) += (api_count as f64 * 0.1).min(9.0);
            evidence.push(format!("API client library ({} refs)", api_count));
        }

        // Fetch API wrapper (709 fetch!)
        let fetch_count = self.code.matches("fetch").count();
        if fetch_count >= 5 {
            *scores.entry(CodeCategory::FetchAPIWrapper).or_insert(0.0) += (fetch_count as f64 * 0.2).min(8.0);
            evidence.push(format!("Fetch API wrapper ({} refs)", fetch_count));
        }
    }

    /// Analyze state management patterns (Iteration 5 discoveries)
    fn analyze_state_patterns(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Action dispatcher (1810 action, 179 dispatch!)
        let action_count = self.code.matches("action").count();
        let dispatch_count = self.code.matches("dispatch").count();
        if action_count >= 10 || dispatch_count >= 5 {
            *scores.entry(CodeCategory::ActionDispatcher).or_insert(0.0) += ((action_count + dispatch_count * 2) as f64 * 0.05).min(10.0);
            evidence.push(format!("Action dispatcher ({} actions, {} dispatch)", action_count, dispatch_count));
        }

        // State selector (168 selector!)
        let selector_count = self.code.matches("selector").count();
        if selector_count >= 5 {
            *scores.entry(CodeCategory::StateSelector).or_insert(0.0) += (selector_count as f64 * 0.3).min(8.0);
            evidence.push(format!("State selector ({} refs)", selector_count));
        }

        // Store manager (410 store!)
        let store_count = self.code.matches("store").count();
        if store_count >= 5 {
            *scores.entry(CodeCategory::StoreManagerCore).or_insert(0.0) += (store_count as f64 * 0.2).min(8.0);
            evidence.push(format!("Store manager ({} refs)", store_count));
        }
    }

    /// Analyze editor features (Iteration 5 discoveries)
    fn analyze_editor_features(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Diff viewer (601 diff!)
        let diff_count = self.code.matches("diff").count();
        if diff_count >= 5 {
            *scores.entry(CodeCategory::DiffViewerComponent).or_insert(0.0) += (diff_count as f64 * 0.3).min(9.0);
            evidence.push(format!("Diff viewer ({} refs)", diff_count));
        }

        // Merge conflict resolver (635 merge, 62 conflict!)
        let merge_count = self.code.matches("merge").count();
        let conflict_count = self.code.matches("conflict").count();
        if merge_count >= 5 || conflict_count >= 3 {
            *scores.entry(CodeCategory::MergeConflictResolver).or_insert(0.0) += ((merge_count + conflict_count * 2) as f64 * 0.1).min(9.0);
            evidence.push(format!("Merge resolver ({} merge, {} conflict)", merge_count, conflict_count));
        }

        // Compact operation (353 compact!)
        let compact_count = self.code.matches("compact").count();
        if compact_count >= 5 {
            *scores.entry(CodeCategory::CompactOperationManager).or_insert(0.0) += (compact_count as f64 * 0.3).min(8.0);
            evidence.push(format!("Compact operations ({} refs)", compact_count));
        }

        // Teleport navigator (112 teleport!)
        let teleport_count = self.code.matches("teleport").count();
        if teleport_count >= 3 {
            *scores.entry(CodeCategory::TeleportNavigator).or_insert(0.0) += (teleport_count as f64 * 0.5).min(8.0);
            evidence.push(format!("Teleport navigator ({} refs)", teleport_count));
        }
    }

    /// Analyze service architecture (Iteration 5 discoveries)
    fn analyze_services(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Plane service (188 PlaneService!)
        let plane_count = self.code.matches("PlaneService").count();
        if plane_count >= 3 {
            *scores.entry(CodeCategory::PlaneServiceCoordinator).or_insert(0.0) += (plane_count as f64 * 0.5).min(9.0);
            evidence.push(format!("Plane service ({} refs)", plane_count));
        }

        // Credentials provider (112 CredentialsProvider!)
        let creds_count = self.code.matches("CredentialsProvider").count();
        if creds_count >= 3 {
            *scores.entry(CodeCategory::CredentialsProviderSystem).or_insert(0.0) += (creds_count as f64 * 0.5).min(9.0);
            evidence.push(format!("Credentials provider ({} refs)", creds_count));
        }
    }

    /// Analyze specific error types (Iteration 6 discoveries)
    fn analyze_error_types(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // TypeError (1457 occurrences!)
        let type_error_count = self.code.matches("TypeError").count();
        if type_error_count >= 3 {
            *scores.entry(CodeCategory::TypeErrorHandler).or_insert(0.0) += (type_error_count as f64 * 0.5).min(10.0);
            evidence.push(format!("TypeError handler ({} errors)", type_error_count));
        }

        // ParameterError (350 occurrences!)
        let param_error_count = self.code.matches("ParameterError").count();
        if param_error_count >= 2 {
            *scores.entry(CodeCategory::ParameterErrorHandler).or_insert(0.0) += (param_error_count as f64 * 0.7).min(9.0);
            evidence.push(format!("ParameterError handler ({} errors)", param_error_count));
        }

        // ProviderError (156 occurrences!)
        let provider_error_count = self.code.matches("ProviderError").count();
        if provider_error_count >= 2 {
            *scores.entry(CodeCategory::ProviderErrorHandler).or_insert(0.0) += (provider_error_count as f64 * 0.8).min(9.0);
            evidence.push(format!("ProviderError handler ({} errors)", provider_error_count));
        }

        // RangeError (144 occurrences!)
        let range_error_count = self.code.matches("RangeError").count();
        if range_error_count >= 2 {
            *scores.entry(CodeCategory::RangeErrorHandler).or_insert(0.0) += (range_error_count as f64 * 0.8).min(8.0);
            evidence.push(format!("RangeError handler ({} errors)", range_error_count));
        }

        // ServiceException (112 occurrences!)
        let service_exc_count = self.code.matches("ServiceException").count();
        if service_exc_count >= 2 {
            *scores.entry(CodeCategory::ServiceExceptionHandler).or_insert(0.0) += (service_exc_count as f64 * 0.8).min(8.0);
            evidence.push(format!("ServiceException handler ({} errors)", service_exc_count));
        }

        // Iteration 8: Additional error types

        // AbortError (88 occurrences!)
        let abort_error_count = self.code.matches("AbortError").count();
        if abort_error_count >= 2 {
            *scores.entry(CodeCategory::AbortErrorHandler).or_insert(0.0) += (abort_error_count as f64 * 0.8).min(8.0);
            evidence.push(format!("AbortError handler ({} errors)", abort_error_count));
        }

        // SyntaxError (82 occurrences!)
        let syntax_error_count = self.code.matches("SyntaxError").count();
        if syntax_error_count >= 2 {
            *scores.entry(CodeCategory::SyntaxErrorHandler).or_insert(0.0) += (syntax_error_count as f64 * 0.8).min(8.0);
            evidence.push(format!("SyntaxError handler ({} errors)", syntax_error_count));
        }

        // TimeoutError (76 occurrences!)
        let timeout_error_count = self.code.matches("TimeoutError").count();
        if timeout_error_count >= 2 {
            *scores.entry(CodeCategory::TimeoutErrorHandler).or_insert(0.0) += (timeout_error_count as f64 * 0.8).min(8.0);
            evidence.push(format!("TimeoutError handler ({} errors)", timeout_error_count));
        }

        // UnknownError (62 occurrences!)
        let unknown_error_count = self.code.matches("UnknownError").count();
        if unknown_error_count >= 2 {
            *scores.entry(CodeCategory::UnknownErrorHandler).or_insert(0.0) += (unknown_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("UnknownError handler ({} errors)", unknown_error_count));
        }

        // QueryError (62 occurrences!)
        let query_error_count = self.code.matches("QueryError").count();
        if query_error_count >= 2 {
            *scores.entry(CodeCategory::QueryErrorHandler).or_insert(0.0) += (query_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("QueryError handler ({} errors)", query_error_count));
        }

        // ReferenceError (59 occurrences!)
        let ref_error_count = self.code.matches("ReferenceError").count();
        if ref_error_count >= 2 {
            *scores.entry(CodeCategory::ReferenceErrorHandler).or_insert(0.0) += (ref_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("ReferenceError handler ({} errors)", ref_error_count));
        }

        // ParseError (58 occurrences!)
        let parse_error_count = self.code.matches("ParseError").count();
        if parse_error_count >= 2 {
            *scores.entry(CodeCategory::ParseErrorHandler).or_insert(0.0) += (parse_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("ParseError handler ({} errors)", parse_error_count));
        }

        // ResponseError (54 occurrences!)
        let response_error_count = self.code.matches("ResponseError").count();
        if response_error_count >= 2 {
            *scores.entry(CodeCategory::ResponseErrorHandler).or_insert(0.0) += (response_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("ResponseError handler ({} errors)", response_error_count));
        }

        // RequestError (54 occurrences!)
        let request_error_count = self.code.matches("RequestError").count();
        if request_error_count >= 2 {
            *scores.entry(CodeCategory::RequestErrorHandler).or_insert(0.0) += (request_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("RequestError handler ({} errors)", request_error_count));
        }

        // InternalError (54 occurrences!)
        let internal_error_count = self.code.matches("InternalError").count();
        if internal_error_count >= 2 {
            *scores.entry(CodeCategory::InternalErrorHandler).or_insert(0.0) += (internal_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("InternalError handler ({} errors)", internal_error_count));
        }

        // TokenException (48 occurrences!)
        let token_exc_count = self.code.matches("TokenException").count();
        if token_exc_count >= 2 {
            *scores.entry(CodeCategory::TokenExceptionHandler).or_insert(0.0) += (token_exc_count as f64 * 0.7).min(7.0);
            evidence.push(format!("TokenException handler ({} errors)", token_exc_count));
        }

        // ServerException (44 occurrences!)
        let server_exc_count = self.code.matches("ServerException").count();
        if server_exc_count >= 2 {
            *scores.entry(CodeCategory::ServerExceptionHandler).or_insert(0.0) += (server_exc_count as f64 * 0.7).min(7.0);
            evidence.push(format!("ServerException handler ({} errors)", server_exc_count));
        }

        // AxiosError (44 occurrences!)
        let axios_error_count = self.code.matches("AxiosError").count();
        if axios_error_count >= 2 {
            *scores.entry(CodeCategory::AxiosErrorHandler).or_insert(0.0) += (axios_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("AxiosError handler ({} errors)", axios_error_count));
        }

        // ParserError (42 occurrences!)
        let parser_error_count = self.code.matches("ParserError").count();
        if parser_error_count >= 2 {
            *scores.entry(CodeCategory::ParserErrorHandler).or_insert(0.0) += (parser_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("ParserError handler ({} errors)", parser_error_count));
        }

        // AuthError (42 occurrences!)
        let auth_error_count = self.code.matches("AuthError").count();
        if auth_error_count >= 2 {
            *scores.entry(CodeCategory::AuthErrorHandler).or_insert(0.0) += (auth_error_count as f64 * 0.7).min(7.0);
            evidence.push(format!("AuthError handler ({} errors)", auth_error_count));
        }
    }

    /// ITERATION 13: Detect content-based patterns that defeat filename obfuscation
    /// These patterns prioritize code content over misleading source map names
    fn analyze_content_based_categories(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // 1. Elliptic Curve Cryptography (HmacDRBG, EC, KeyPair, Signature)
        let has_hmac_drbg = self.code.contains("HmacDRBG");
        let has_keypair = self.code.contains("KeyPair") || self.code.contains("keyPair");
        let has_ec = self.code.matches("function EC(").count() > 0 || self.code.contains("new EC(");
        let has_signature = self.code.contains("Signature");
        let has_curve = self.code.contains(".curve") && self.code.contains("precompute");

        if (has_hmac_drbg && has_keypair) || (has_ec && has_curve) {
            let mut crypto_score = 10.0;
            // HmacDRBG is EXTREMELY specific to elliptic curve crypto
            if has_hmac_drbg {
                crypto_score += 0.6;
            }
            // Multiple crypto signals = very confident
            if has_hmac_drbg && has_keypair && has_signature {
                crypto_score += 0.4;
            }
            *scores.entry(CodeCategory::EllipticCurveCrypto).or_insert(0.0) += crypto_score;
            evidence.push(format!("Elliptic Curve Cryptography (score: {:.1})", crypto_score));
        }

        // 2. Node Error Factory (createErrorType, NodeError, codes[])
        let has_create_error_type = self.code.contains("createErrorType");
        let has_node_error = self.code.contains("NodeError");
        let has_codes_array = self.code.contains("codes[");
        let has_get_message = self.code.contains("getMessage");

        if has_create_error_type && (has_node_error || has_codes_array) {
            let mut error_factory_score = 10.0;
            // createErrorType is very specific to Node.js error factory pattern
            if has_create_error_type && has_node_error {
                error_factory_score += 0.5;
            }
            *scores.entry(CodeCategory::NodeErrorFactory).or_insert(0.0) += error_factory_score;
            evidence.push(format!("Node.js Error Factory (score: {:.1})", error_factory_score));
        }

        // 3. Bitwise Crypto Operations (ushrn, bitLength, nh)
        let has_ushrn = self.code.contains("ushrn");
        let has_bit_length = self.code.contains("bitLength");
        let has_nh = self.code.contains(".nh") && self.code.contains("result");
        let has_bit_ops = self.code.contains("<<") || self.code.contains(">>");

        if (has_ushrn && has_bit_length) || (has_nh && has_bit_ops) {
            *scores.entry(CodeCategory::BitwiseCryptoOps).or_insert(0.0) += 10.0;
            evidence.push("Bitwise Crypto Operations (ushrn, bitLength)".to_string());
        }

        // 4. JavaScript Syntax Highlighter (JSX, className, keyword highlighting)
        let has_jsx_tags = self.code.contains("begin:\"<>\"") || self.code.contains("end:\"</>\"");
        let has_classname = self.code.contains("className:\"number\"") || self.code.contains("className:\"string\"");
        let has_keyword_literal = self.code.contains("keyword:") && self.code.contains("literal:");
        let has_builtin = self.code.contains("built_in:");
        let has_uri_encode = self.code.contains("decodeURI") && self.code.contains("encodeURI");

        if (has_jsx_tags && has_classname) || (has_keyword_literal && has_builtin) || has_uri_encode {
            *scores.entry(CodeCategory::JavaScriptSyntaxHighlighter).or_insert(0.0) += 10.0;
            evidence.push("JavaScript Syntax Highlighter (JSX, keywords, className)".to_string());
        }

        // 5. React DevTools Profiler (__REACT_DEVTOOLS_GLOBAL_HOOK__, performance.now)
        let has_devtools_hook = self.code.contains("__REACT_DEVTOOLS_GLOBAL_HOOK__");
        let has_get_internal_ranges = self.code.contains("getInternalModuleRanges");
        let has_display_name_fiber = self.code.contains("getDisplayNameForFiber");
        let has_is_profiling = self.code.contains("getIsProfiling");
        let has_profiler_version = self.code.contains("--profiler-v") || self.code.contains("--react-version");

        if has_devtools_hook || (has_display_name_fiber && has_is_profiling) || has_profiler_version {
            let mut devtools_score = 10.0;
            // __REACT_DEVTOOLS_GLOBAL_HOOK__ is EXTREMELY unique
            if has_devtools_hook {
                devtools_score += 0.7;
            }
            *scores.entry(CodeCategory::ReactDevToolsProfiler).or_insert(0.0) += devtools_score;
            evidence.push(format!("React DevTools Profiler (score: {:.1})", devtools_score));
        }

        // 6. API Error Handler (API-specific error patterns)
        let api_error_patterns = self.code.matches("api_error").count() +
                                 self.code.matches("ApiError").count() +
                                 self.code.matches("API_ERROR").count();
        if api_error_patterns >= 3 {
            *scores.entry(CodeCategory::APIErrorHandler).or_insert(0.0) += 8.0;
            evidence.push(format!("API Error Handler ({} patterns)", api_error_patterns));
        }

        // 7. Error Recovery System (recovery strategies, retry logic)
        let has_recovery = self.code.contains("recovery") || self.code.contains("recover");
        let has_retry = self.code.contains("retry") || self.code.contains("Retry");
        let has_fallback = self.code.contains("fallback") || self.code.contains("Fallback");

        if (has_recovery && has_retry) || (has_recovery && has_fallback) {
            *scores.entry(CodeCategory::ErrorRecoverySystem).or_insert(0.0) += 8.0;
            evidence.push("Error Recovery System (recovery, retry, fallback)".to_string());
        }

        // 8. Fallback Error Handler
        let fallback_count = self.code.matches("fallback").count();
        if fallback_count >= 5 {
            *scores.entry(CodeCategory::FallbackErrorHandler).or_insert(0.0) += 8.0;
            evidence.push(format!("Fallback Error Handler ({} refs)", fallback_count));
        }

        // 9. Promise Error Handler (unhandledRejection, promise rejection)
        let has_unhandled_rejection = self.code.contains("unhandledRejection");
        let has_promise_reject = self.code.contains("PromiseRejection") ||
                                self.code.contains("promise.reject");
        let promise_error_count = self.code.matches("promiseError").count();

        if has_unhandled_rejection || has_promise_reject || promise_error_count >= 3 {
            *scores.entry(CodeCategory::PromiseErrorHandler).or_insert(0.0) += 8.0;
            evidence.push("Promise Error Handler (unhandledRejection)".to_string());
        }

        // 10. Regex Engine (regex compilation and execution)
        let has_regex_compile = self.code.contains("RegExp(") || self.code.contains("new RegExp");
        let has_regex_test = self.code.contains(".test(") && has_regex_compile;
        let has_regex_exec = self.code.contains(".exec(") && has_regex_compile;
        let regex_count = self.code.matches("RegExp").count();

        if (has_regex_test || has_regex_exec) && regex_count >= 3 {
            *scores.entry(CodeCategory::RegexEngine).or_insert(0.0) += 8.0;
            evidence.push(format!("Regex Engine ({} RegExp patterns)", regex_count));
        }

        // 11. Timestamp Manager (timestamp creation and formatting)
        let has_timestamp = self.code.matches("timestamp").count();
        let has_to_iso = self.code.contains("toISOString");
        let has_date_now = self.code.contains("Date.now");

        if has_timestamp >= 5 || (has_to_iso && has_date_now) {
            *scores.entry(CodeCategory::TimestampManager).or_insert(0.0) += 7.0;
            evidence.push(format!("Timestamp Manager ({} refs)", has_timestamp));
        }

        // 12. Sync File I/O (sync file operations)
        let sync_ops = self.code.matches("Sync").count();
        let has_fs_sync = self.code.contains("readFileSync") ||
                         self.code.contains("writeFileSync") ||
                         self.code.contains("existsSync");

        if has_fs_sync && sync_ops >= 3 {
            *scores.entry(CodeCategory::SyncFileIO).or_insert(0.0) += 8.0;
            evidence.push(format!("Sync File I/O ({} sync operations)", sync_ops));
        }

        // 13. Image Processor (image handling)
        let has_image = self.code.matches("image").count() + self.code.matches("Image").count();
        let has_canvas = self.code.contains("canvas") || self.code.contains("Canvas");
        let has_png_jpg = self.code.contains("png") || self.code.contains("jpg") || self.code.contains("jpeg");

        if has_image >= 5 || (has_canvas && has_png_jpg) {
            *scores.entry(CodeCategory::ImageProcessor).or_insert(0.0) += 7.0;
            evidence.push(format!("Image Processor ({} image refs)", has_image));
        }

        // ITERATION 14: Additional obfuscated utilities!

        // 14. Date-fns Library (date manipulation with locale support)
        let has_week_starts_on = self.code.contains("weekStartsOn");
        let has_get_full_year = self.code.contains("getFullYear");
        let has_date_utc = self.code.contains("Date.UTC");
        let has_locale_options = self.code.contains("locale?.options");
        let has_set_hours_zero = self.code.contains("setHours(0,0,0,0)");

        if (has_week_starts_on && has_get_full_year) || (has_date_utc && has_set_hours_zero) || has_locale_options {
            *scores.entry(CodeCategory::DateFnsLibrary).or_insert(0.0) += 10.0;
            evidence.push("Date-fns Library (weekStartsOn, locale-aware dates)".to_string());
        }

        // 15. Debounce/Throttle (Lodash-style timer control)
        let has_leading = self.code.contains("leading");
        let has_trailing = self.code.contains("trailing");
        let has_max_wait = self.code.contains("maxWait");
        let has_clear_timeout = self.code.contains("clearTimeout");
        let debounce_count = self.code.matches("debounce").count() + self.code.matches("throttle").count();

        if (has_leading && has_trailing && has_max_wait) || debounce_count >= 3 {
            *scores.entry(CodeCategory::DebounceThrottle).or_insert(0.0) += 10.0;
            evidence.push("Debounce/Throttle (leading, trailing, maxWait)".to_string());
        }

        // 16. JSON Tokenizer (brace/bracket matching for parsing)
        let has_type_brace = self.code.contains("type:\"brace\"") || self.code.contains("type:'brace'");
        let has_type_paren = self.code.contains("type:\"paren\"") || self.code.contains("type:'paren'");
        let has_json_parse = self.code.contains("JSON.parse");
        let bracket_count = self.code.matches("{type:").count();

        if (has_type_brace && has_type_paren) || (bracket_count >= 3 && has_json_parse) {
            *scores.entry(CodeCategory::JSONTokenizer).or_insert(0.0) += 10.0;
            evidence.push("JSON Tokenizer (brace matching, token types)".to_string());
        }

        // 17. Startup Profiler (performance timing and reporting)
        let has_startup_time = self.code.contains("startup") && self.code.contains("startTime");
        let has_perf_marks = self.code.contains("getEntriesByType(\"mark\")") ||
                            self.code.contains("getEntriesByType('mark')");
        let has_profiling = self.code.contains("profiling") || self.code.contains("Performance");
        let has_startup_perf = self.code.contains("startup-perf");

        if has_startup_perf || (has_startup_time && has_perf_marks) || (has_perf_marks && has_profiling) {
            let mut profiler_score = 10.0;
            // "startup-perf" path is very specific
            if has_startup_perf {
                profiler_score += 0.6;
            }
            *scores.entry(CodeCategory::StartupProfiler).or_insert(0.0) += profiler_score;
            evidence.push(format!("Startup Profiler (score: {:.1})", profiler_score));
        }

        // 18. Object Inspector (property introspection and serialization)
        let has_get_own_property = self.code.contains("getOwnPropertyDescriptor") ||
                                  self.code.contains("getOwnPropertyNames");
        let has_proto_check = self.code.contains("__proto__");
        let has_constructor_name = self.code.contains("constructor?.name") ||
                                   self.code.contains("constructor.name");
        let has_to_iso_string = self.code.contains("toISOString()");

        if (has_get_own_property && has_proto_check) ||
           (has_constructor_name && has_to_iso_string && has_proto_check) {
            *scores.entry(CodeCategory::ObjectInspector).or_insert(0.0) += 10.0;
            evidence.push("Object Inspector (property introspection, __proto__)".to_string());
        }

        // 19. Elm Syntax Highlighter (Elm language highlighting)
        let has_infix_keywords = self.code.contains("infix infixl infixr") ||
                                self.code.contains("infixl") ||
                                self.code.contains("infixr");
        let has_port_keyword = self.code.contains("port") && self.code.contains("keywords");
        let has_illegal_semicolon = self.code.contains("illegal:/;/") ||
                                    self.code.contains("illegal: /;/");

        if (has_infix_keywords && has_port_keyword) || has_illegal_semicolon {
            let mut elm_score = 10.0;
            // Uniqueness bonus: illegal:/;/ is VERY Elm-specific
            if has_illegal_semicolon {
                elm_score += 0.5;  // Tie-breaker bonus
            }
            // Multiple signals bonus
            if has_infix_keywords && has_port_keyword && has_illegal_semicolon {
                elm_score += 0.3;  // Extra confidence
            }
            *scores.entry(CodeCategory::ElmSyntaxHighlighter).or_insert(0.0) += elm_score;
            evidence.push(format!("Elm Syntax Highlighter (score: {:.1})", elm_score));
        }

        // 20. Lodash Type Checker (type introspection with [object Type] tags)
        let object_type_count = self.code.matches("[object ").count();
        let has_object_array = self.code.contains("[object Array]");
        let has_object_function = self.code.contains("[object Function]");
        let has_object_regexp = self.code.contains("[object RegExp]");
        let has_max_safe_int = self.code.contains("9007199254740991");

        if object_type_count >= 5 || (has_object_array && has_object_function) || has_max_safe_int {
            *scores.entry(CodeCategory::LodashTypeChecker).or_insert(0.0) += 10.0;
            evidence.push(format!("Lodash Type Checker ({} type tags)", object_type_count));
        }

        // 21. RxJS Operators (.subscribe, createOperatorSubscriber, Observable)
        let has_subscribe = self.code.contains(".subscribe(") || self.code.contains(".unsubscribe()");
        let has_observable = self.code.contains("Observable") || self.code.contains("Subject");
        let has_operator = self.code.contains("createOperatorSubscriber") ||
                           self.code.contains(".operate(function");
        let subscribe_count = self.code.matches("subscribe").count();

        if (has_subscribe && has_observable) || subscribe_count >= 3 {
            let mut rxjs_score = 10.0;
            // createOperatorSubscriber is extremely specific to RxJS
            if has_operator {
                rxjs_score += 0.5;
            }
            // Multiple subscribe patterns = reactive programming
            if subscribe_count >= 5 {
                rxjs_score += 0.3;
            }
            *scores.entry(CodeCategory::RxJSOperators).or_insert(0.0) += rxjs_score;
            evidence.push(format!("RxJS Operators (score: {:.1})", rxjs_score));
        }

        // 22. OpenTelemetry Encoding (hrTimeToNanos, hexToBinary, encodeAsLongBits)
        let has_hex_to_binary = self.code.contains("hexToBinary");
        let has_hr_time = self.code.contains("hrTimeToNanos") || self.code.contains("encodeAsLongBits");
        let has_instrumentation = self.code.contains("createInstrumentationScope") ||
                                  self.code.contains("createResource");
        let has_to_any_value = self.code.contains("toAnyValue") || self.code.contains("toAttributes");
        let has_bigint_encoding = self.code.contains("BigInt.asUintN") && self.code.contains("low:") && self.code.contains("high:");

        if (has_hex_to_binary && has_hr_time) || has_instrumentation || (has_bigint_encoding && has_to_any_value) {
            let mut otel_score = 10.0;
            // hrTimeToNanos is extremely specific to OpenTelemetry
            if has_hr_time {
                otel_score += 0.6;
            }
            // Multiple signals
            if has_hex_to_binary && has_hr_time && has_instrumentation {
                otel_score += 0.3;
            }
            *scores.entry(CodeCategory::OpenTelemetryEncoding).or_insert(0.0) += otel_score;
            evidence.push(format!("OpenTelemetry Encoding (score: {:.1})", otel_score));
        }

        // 23. Zlib Compression (Z_ constants, deflate/inflate)
        let has_z_constants = self.code.contains("Z_MIN_WINDOWBITS") ||
                              self.code.contains("Z_DEFAULT_CHUNK") ||
                              self.code.contains("Z_MAX_LEVEL");
        let has_deflate = self.code.contains("deflate") || self.code.contains("inflate");
        let has_gzip = self.code.contains("gzip") || self.code.contains("gunzip");
        // PRECISION FIX: Only count ACTUAL zlib constants, not just "Z_" anywhere
        let zlib_specific_constants = [
            "Z_NO_FLUSH", "Z_PARTIAL_FLUSH", "Z_SYNC_FLUSH", "Z_FULL_FLUSH",
            "Z_FINISH", "Z_BLOCK", "Z_TREES", "Z_OK", "Z_STREAM_END",
            "Z_NEED_DICT", "Z_ERRNO", "Z_STREAM_ERROR", "Z_DATA_ERROR",
            "Z_BEST_SPEED", "Z_BEST_COMPRESSION", "Z_DEFAULT_COMPRESSION",
            "Z_FILTERED", "Z_HUFFMAN_ONLY", "Z_DEFLATED"
        ];
        let z_count = zlib_specific_constants.iter()
            .filter(|c| self.code.contains(*c))
            .count();

        if has_z_constants || z_count >= 5 {
            let mut zlib_score = 10.0;
            // Many Z_ constants = definitely zlib
            if z_count >= 10 {
                zlib_score += 0.7;
            }
            // deflate/inflate operations
            if has_deflate || has_gzip {
                zlib_score += 0.2;
            }
            *scores.entry(CodeCategory::ZlibCompression).or_insert(0.0) += zlib_score;
            evidence.push(format!("Zlib Compression (score: {:.1})", zlib_score));
        }

        // 24. Installation Detection (Homebrew, winget, npm config)
        let has_homebrew = self.code.contains("Homebrew") || self.code.contains("Caskroom");
        let has_winget = self.code.contains("winget") || self.code.contains("Microsoft/WinGet");
        let has_npm_prefix = self.code.contains("npm config get prefix");
        let has_exec_path = self.code.contains("process.execPath");
        let has_install_paths = self.code.contains("node_modules") ||
                                self.code.contains("/opt/homebrew/") ||
                                self.code.contains(".nvm/versions");

        if (has_homebrew && has_exec_path) || (has_winget && has_exec_path) || has_npm_prefix {
            let mut install_score = 10.0;
            // Multi-platform detection
            if has_homebrew && has_winget {
                install_score += 0.4;
            }
            // Installation path detection
            if has_install_paths && has_exec_path {
                install_score += 0.3;
            }
            *scores.entry(CodeCategory::InstallationDetection).or_insert(0.0) += install_score;
            evidence.push(format!("Installation Detection (score: {:.1})", install_score));
        }

        // 25. Anthropic API Client (api.anthropic.com, CLAUDE_CODE_USE_BEDROCK)
        let has_api_anthropic = self.code.contains("api.anthropic.com");
        let has_bedrock = self.code.contains("CLAUDE_CODE_USE_BEDROCK") ||
                          self.code.contains("CLAUDE_CODE_USE_VERTEX") ||
                          self.code.contains("CLAUDE_CODE_USE_FOUNDRY");
        let has_session_logs = self.code.contains("Error fetching session logs") ||
                               self.code.contains("session_get_fail_status");
        let has_connectivity = self.code.contains("isConnected") && self.code.contains("EHOSTUNREACH");

        if has_api_anthropic || (has_bedrock && has_connectivity) {
            let mut api_score = 10.0;
            // api.anthropic.com is extremely specific
            if has_api_anthropic {
                api_score += 0.7;
            }
            // Multiple signals
            if has_bedrock && has_session_logs {
                api_score += 0.2;
            }
            *scores.entry(CodeCategory::AnthropicAPIClient).or_insert(0.0) += api_score;
            evidence.push(format!("Anthropic API Client (score: {:.1})", api_score));
        }

        // 26. Lodash Core Library (heavy typeof checks, Object.keys, type coercion)
        let typeof_count = self.code.matches("typeof").count();
        let has_object_keys = self.code.contains("Object.keys") ||
                              self.code.contains("Object.getOwnPropertySymbols");
        let has_value_of = self.code.contains("T.valueOf==") ||
                           self.code.contains("$.valueOf()") ||
                           self.code.contains("S.valueOf");
        let has_type_coercion = self.code.contains("T===0?T:0") ||
                                self.code.contains("S===S?") ||
                                self.code.contains("T===T?T:0");
        let minified_function_count = self.code.matches("function").count();
        let single_char_params = self.code.matches("(T,S)").count() +
                                 self.code.matches("(T)").count();

        if (typeof_count >= 5 && has_object_keys && has_type_coercion) ||
           (minified_function_count >= 10 && single_char_params >= 5) {
            let mut lodash_score = 10.0;
            // valueOf + type coercion is very Lodash-specific
            if has_value_of && has_type_coercion {
                lodash_score += 0.4;
            }
            // Heavy minification with single-char params
            if single_char_params >= 10 {
                lodash_score += 0.3;
            }
            *scores.entry(CodeCategory::LodashCoreLibrary).or_insert(0.0) += lodash_score;
            evidence.push(format!("Lodash Core Library (score: {:.1})", lodash_score));
        }

        // 27. Crypto Library Wrappers (Iteration 24: browserify-crypto, ASN.1, elliptic, hash libs)
        // These are thin wrappers that re-export crypto functions
        let has_crypto_require = self.code.contains("require(\"crypto\")");
        let has_crypto_exports = self.code.contains("createCipher") ||
                                 self.code.contains("createCipheriv") ||
                                 self.code.contains("createHash") ||
                                 self.code.contains("createHmac") ||
                                 self.code.contains("createSign") ||
                                 self.code.contains("createVerify") ||
                                 self.code.contains("DiffieHellman") ||
                                 self.code.contains("createECDH");

        // Browserify crypto wrappers
        let has_browserify_crypto = (self.code.contains("browserify") ||
                                     self.code.contains("Browserify")) &&
                                    (self.code.contains("Cipher") ||
                                     self.code.contains("Hash") ||
                                     self.code.contains("Sign"));

        // ASN.1 library structure (asn1.encoders, asn1.decoders, asn1.bignum)
        let has_asn1 = self.code.contains("asn1.") &&
                       (self.code.contains(".encoders") ||
                        self.code.contains(".decoders") ||
                        self.code.contains(".bignum") ||
                        self.code.contains(".define"));

        // Elliptic curve library entry point
        let has_elliptic_lib = self.code.contains("elliptic.") &&
                               (self.code.contains(".curve") ||
                                self.code.contains(".ec") ||
                                self.code.contains(".eddsa") ||
                                self.code.contains(".utils"));

        // Hash library entry point (hash.sha1, hash.sha256, hash.ripemd160)
        let has_hash_lib = self.code.contains("hash.") &&
                          (self.code.contains(".sha") ||
                           self.code.contains(".ripemd") ||
                           self.code.contains(".hmac"));

        // OID mappings for crypto algorithms (2.16.840.1.101.3.4.1.X = aes-X)
        let has_crypto_oids = self.code.contains("2.16.840.1.101") &&
                             (self.code.contains("aes-") ||
                              self.code.contains("des-") ||
                              self.code.contains("rsa-"));

        // Module re-export pattern: exports.X = crypto.X or exports.X = require_X()
        let has_crypto_reexport = (self.code.contains("exports.") ||
                                   self.code.contains("module.exports")) &&
                                  (self.code.contains("Cipher") ||
                                   self.code.contains("Hash") ||
                                   self.code.contains("Hmac") ||
                                   self.code.contains("Sign") ||
                                   self.code.contains("Verify") ||
                                   self.code.contains("ECDH"));

        if (has_crypto_require && has_crypto_exports) ||
           has_browserify_crypto ||
           has_asn1 ||
           has_elliptic_lib ||
           has_hash_lib ||
           has_crypto_oids ||
           (has_crypto_reexport && has_crypto_exports) {
            let mut crypto_score = 10.0;

            // Uniqueness bonuses for highly specific patterns
            if has_asn1 || has_elliptic_lib || has_hash_lib {
                crypto_score += 0.6;  // Library entry points are very specific
            }
            if has_crypto_oids {
                crypto_score += 0.5;  // OID mappings are extremely unique
            }
            if has_browserify_crypto {
                crypto_score += 0.4;  // Browserify wrappers are specific
            }

            *scores.entry(CodeCategory::CryptoLibraryWrappers).or_insert(0.0) += crypto_score;
            evidence.push(format!("Crypto Library Wrappers (score: {:.1})", crypto_score));
        }
    }

    /// Detect syntax highlighters and language parsers (HIGH PRIORITY - prevents false positives)
    fn analyze_syntax_highlighter(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let language_indicators = [
            ("name:", "keywords:", "aliases:"),  // Syntax highlighter structure
            ("grammar", "tokenizer", "syntax"),
            ("lexer", "parser", "AST"),
            ("language:", "mode:", "syntax:"),
        ];

        // Check for language definition patterns
        let has_language_structure = language_indicators.iter()
            .any(|(p1, p2, p3)| {
                [*p1, *p2, *p3].iter()
                    .filter(|p| self.code.contains(**p))
                    .count() >= 2
            });

        // Check for specific language names in keyword lists
        let language_keywords = [
            "JavaScript", "TypeScript", "Python", "Ruby", "Java", "C++",
            "XQuery", "XPath", "XML", "SQL", "HTML", "CSS", "JSON",
            "keyword:", "comment:", "string:", "number:", "operator:",
        ];

        let keyword_count = language_keywords.iter()
            .filter(|k| self.code.contains(*k))
            .count();

        if has_language_structure && keyword_count >= 3 {
            *scores.entry(CodeCategory::SyntaxHighlighter).or_insert(0.0) += 3.0;  // High score to override others
            evidence.push(format!("Syntax highlighter (detected {} language keywords)", keyword_count));
        } else if has_language_structure {
            *scores.entry(CodeCategory::LanguageParser).or_insert(0.0) += 2.5;
            evidence.push("Language parser structure".to_string());
        }

        // Detect common highlighter libraries
        if self.code.contains("highlight.js") || self.code.contains("hljs") {
            *scores.entry(CodeCategory::SyntaxHighlighter).or_insert(0.0) += 2.0;
            evidence.push("highlight.js library".to_string());
        }
        if self.code.contains("Prism") && self.code.contains("language-") {
            *scores.entry(CodeCategory::SyntaxHighlighter).or_insert(0.0) += 2.0;
            evidence.push("Prism syntax highlighter".to_string());
        }
    }

    /// Detect state machines (multiple conditional branches on state)
    fn analyze_state_machine(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // Pattern: if (state === X) ... else if (state === Y) ... else ...
        let state_pattern = Regex::new(r#"if\s*\([^)]*\.(state|mode|status|phase)\s*===?\s*["']([^"']+)["']\)"#).unwrap();
        let matches: Vec<_> = state_pattern.captures_iter(&self.code).collect();

        if matches.len() >= 3 {
            let states: Vec<String> = matches.iter()
                .filter_map(|m| m.get(2).map(|s| s.as_str().to_string()))
                .collect();

            *scores.entry(CodeCategory::StateManagement).or_insert(0.0) += matches.len() as f64 * 0.3;
            evidence.push(format!("State machine with {} states: {:?}", states.len(), states));
        }

        // Pattern: switch(type) with multiple cases
        let switch_pattern = Regex::new(r"switch\s*\([^)]*\.type\)").unwrap();
        let case_pattern = Regex::new(r#"case\s+["']([^"']+)["']:"#).unwrap();

        if switch_pattern.is_match(&self.code) {
            let cases: Vec<_> = case_pattern.captures_iter(&self.code)
                .filter_map(|m| m.get(1).map(|s| s.as_str().to_string()))
                .collect();

            if cases.len() >= 3 {
                *scores.entry(CodeCategory::EventHandling).or_insert(0.0) += cases.len() as f64 * 0.4;
                evidence.push(format!("Event handler with {} event types", cases.len()));
            }
        }
    }

    /// Detect validation logic (error types, schema checks)
    fn analyze_validation(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let validation_errors = [
            "invalid_type", "invalid_value", "too_small", "too_big",
            "invalid_format", "invalid_email", "invalid_url",
            "required", "optional", "nullable",
        ];

        let matches = validation_errors.iter()
            .filter(|e| self.code.contains(*e))
            .count();

        if matches >= 3 {
            *scores.entry(CodeCategory::DataValidation).or_insert(0.0) += matches as f64 * 0.5;
            evidence.push(format!("Schema validation with {} error types", matches));
        }

        // Detect zod/yup/joi patterns
        if self.code.contains("z.object") || self.code.contains("yup.object") || self.code.contains("Joi.object") {
            *scores.entry(CodeCategory::DataValidation).or_insert(0.0) += 1.0;
            evidence.push("Schema validator (zod/yup/joi)".to_string());
        }
    }

    /// Detect authentication flows - now detects DOMAIN-SPECIFIC patterns
    fn analyze_authentication(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // AWS IAM/Cognito patterns
        if (self.code.contains("AccessKeyId") || self.code.contains("SecretAccessKey")) &&
           (self.code.contains("SessionToken") || self.code.contains("credential")) {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 2.0;
            evidence.push("AWS IAM Credentials (AccessKeyId + SecretAccessKey + SessionToken)".to_string());
        }
        if self.code.contains("Cognito") && (self.code.contains("Identity") || self.code.contains("Pool")) {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.5;
            evidence.push("AWS Cognito Identity".to_string());
        }

        // API Key authentication
        if (self.code.contains("apiKey") || self.code.contains("api_key")) &&
           (self.code.contains("header") || self.code.contains("Authorization")) {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.5;
            evidence.push("API Key Authentication".to_string());
        }

        // OAuth flows
        if self.code.contains("OAuth") && self.code.contains("callback") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 2.0;
            evidence.push("OAuth Callback Flow".to_string());
        } else if self.code.contains("OAuth") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.2;
            evidence.push("OAuth Flow".to_string());
        }

        // JWT patterns
        if self.code.contains("JWT") && self.code.contains("refresh") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.8;
            evidence.push("JWT Refresh Token".to_string());
        } else if self.code.contains("JWT") || self.code.contains("jsonwebtoken") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.2;
            evidence.push("JWT Token".to_string());
        }

        // Session-based auth
        if self.code.contains("session") && self.code.contains("cookie") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.5;
            evidence.push("Session Cookie Authentication".to_string());
        }

        // SAML
        if self.code.contains("SAML") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.5;
            evidence.push("SAML Authentication".to_string());
        }

        // MFA/2FA
        if (self.code.contains("MFA") || self.code.contains("2FA") || self.code.contains("twoFactor")) &&
           (self.code.contains("verify") || self.code.contains("code")) {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.5;
            evidence.push("Multi-Factor Authentication".to_string());
        }

        // Social login patterns
        if self.code.contains("google") && self.code.contains("sign") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.0;
            evidence.push("Google Sign-In".to_string());
        }
        if self.code.contains("facebook") && self.code.contains("login") {
            *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += 1.0;
            evidence.push("Facebook Login".to_string());
        }

        // Generic auth patterns (fallback)
        let auth_patterns = [
            ("login", "logout", "session"),
            ("authenticate", "authorize", "credential"),
            ("token", "refresh", "expire"),
        ];

        for (p1, p2, p3) in &auth_patterns {
            let count = [p1, p2, p3].iter()
                .filter(|p| self.code.to_lowercase().contains(&p.to_lowercase()))
                .count();

            if count >= 2 {
                *scores.entry(CodeCategory::AWSCredentialProvider).or_insert(0.0) += count as f64 * 0.4;
                evidence.push(format!("Generic auth pattern: {}/{}/{}", p1, p2, p3));
            }
        }
    }

    /// Detect error handling patterns
    fn analyze_error_handling(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        // ITERATION 25 PRECISION FIX: Require multiple signals to avoid false positives
        // Found: 87% false positive rate with old thresholds!
        // Files with incidental error throwing (React hooks, parsers) were misclassified

        let try_count = self.code.matches("try{").count();
        let catch_count = self.code.matches("catch(").count();
        let throw_count = self.code.matches("throw ").count();
        let error_new_count = self.code.matches("new Error(").count() +
                              self.code.matches("new TypeError(").count() +
                              self.code.matches("new RangeError(").count();

        // Signal 1: Substantial error handling (try/catch blocks)
        let has_error_handling = try_count >= 4 && catch_count >= 4;  // Raised from 2 to 4

        // Signal 2: Substantial error throwing (indicates error generation, not just control flow)
        let has_error_throwing = throw_count >= 8;  // Raised from 3 to 8

        // Signal 3: Error recovery patterns
        let has_error_recovery = self.code.contains("retry") ||
                                 self.code.contains("fallback") ||
                                 self.code.contains("recover");

        // Signal 4: Error construction (creating custom errors)
        let has_error_construction = error_new_count >= 3;

        // Signal 5: Error handler function names
        let has_error_handler_names = self.code.contains("handleError") ||
                                      self.code.contains("errorHandler") ||
                                      self.code.contains("onError") ||
                                      self.code.contains("catchError");

        // Count how many signals are present
        let signal_count = [
            has_error_handling,
            has_error_throwing,
            has_error_recovery,
            has_error_construction,
            has_error_handler_names,
        ].iter().filter(|&&x| x).count();

        // REQUIRE at least 2 signals for error handling to be primary purpose
        if signal_count >= 2 {
            let mut error_score = 0.0;

            if has_error_handling {
                error_score += (try_count + catch_count) as f64 * 0.4;
                evidence.push(format!("Error handling: {} try/catch blocks", try_count));
            }

            if has_error_throwing {
                error_score += throw_count as f64 * 0.3;
                evidence.push(format!("Error throwing: {} throw statements", throw_count));
            }

            if has_error_recovery {
                error_score += 1.5;  // Increased bonus for recovery patterns
                evidence.push("Error recovery (retry/fallback)".to_string());
            }

            if has_error_construction {
                error_score += error_new_count as f64 * 0.4;
                evidence.push(format!("Error construction: {} new Error()", error_new_count));
            }

            if has_error_handler_names {
                error_score += 1.0;
                evidence.push("Error handler function names".to_string());
            }

            // Add bonus for multiple signals (high confidence)
            if signal_count >= 3 {
                error_score += 1.0;
                evidence.push(format!("Multiple error signals ({})", signal_count));
            }

            *scores.entry(CodeCategory::ErrorHandler).or_insert(0.0) += error_score;
        }
    }

    /// Detect telemetry/analytics
    fn analyze_telemetry(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let telemetry_patterns = ["tengu_", "track", "recordEvent", "analytics", "metric"];

        let matches = telemetry_patterns.iter()
            .filter(|p| self.code.contains(*p))
            .count();

        if matches >= 2 {
            *scores.entry(CodeCategory::TelemetryRecorder).or_insert(0.0) += matches as f64 * 0.7;
            evidence.push("Telemetry/analytics tracking".to_string());
        }
    }

    /// Detect permission/access control
    fn analyze_permissions(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let permission_keywords = [
            "MUST NOT", "read-only", "permission", "access control",
            "allowedPaths", "forbidden", "authorized",
        ];

        let matches = permission_keywords.iter()
            .filter(|k| self.code.contains(*k))
            .count();

        if matches >= 2 {
            *scores.entry(CodeCategory::PermissionControl).or_insert(0.0) += matches as f64 * 0.6;
            evidence.push("Permission/access control".to_string());
        }
    }

    /// Detect protocol handling (headers, encoding, parsing)
    fn analyze_protocol(&self, scores: &mut HashMap<CodeCategory, f64>, evidence: &mut Vec<String>) {
        let protocol_patterns = [
            "Claude-", "X-", "Content-Type", "Authorization",
            "header", "encode", "decode", "parse",
        ];

        let matches = protocol_patterns.iter()
            .filter(|p| self.code.contains(*p))
            .count();

        if matches >= 3 {
            *scores.entry(CodeCategory::ProtocolHandler).or_insert(0.0) += matches as f64 * 0.5;
            evidence.push("Protocol/header handling".to_string());
        }
    }

    /// Infer SPECIFIC purpose from category, evidence, and deep context analysis
    fn infer_purpose(&self, category: &CodeCategory, evidence: &[String]) -> String {
        match category {
            CodeCategory::AWSCredentialProvider => {
                // Use evidence to determine specific auth type
                if evidence.iter().any(|e| e.contains("AWS IAM Credentials")) {
                    "AWS IAM Credential Provider".to_string()
                } else if evidence.iter().any(|e| e.contains("AWS Cognito")) {
                    "AWS Cognito Identity Provider".to_string()
                } else if evidence.iter().any(|e| e.contains("API Key Authentication")) {
                    "API Key Auth Handler".to_string()
                } else if evidence.iter().any(|e| e.contains("OAuth Callback")) {
                    "OAuth Callback Handler".to_string()
                } else if evidence.iter().any(|e| e.contains("OAuth Flow")) {
                    "OAuth Flow Controller".to_string()
                } else if evidence.iter().any(|e| e.contains("JWT Refresh")) {
                    "JWT Refresh Token Manager".to_string()
                } else if evidence.iter().any(|e| e.contains("JWT Token")) {
                    "JWT Token Validator".to_string()
                } else if evidence.iter().any(|e| e.contains("Session Cookie")) {
                    "Session Cookie Manager".to_string()
                } else if evidence.iter().any(|e| e.contains("SAML")) {
                    "SAML Authentication Handler".to_string()
                } else if evidence.iter().any(|e| e.contains("Multi-Factor")) {
                    "MFA Verification Handler".to_string()
                } else if evidence.iter().any(|e| e.contains("Google Sign-In")) {
                    "Google OAuth Provider".to_string()
                } else if evidence.iter().any(|e| e.contains("Facebook Login")) {
                    "Facebook OAuth Provider".to_string()
                } else if self.code.contains("login") && self.code.contains("rate") {
                    "Login Rate Limiter".to_string()
                } else {
                    "Auth Flow Controller".to_string()
                }
            },
            CodeCategory::StateManagement => {
                let state_context = self.extract_state_context();
                if state_context.contains("plan") || state_context.contains("Plan") {
                    if state_context.contains("permission") {
                        "Plan Permission State Controller".to_string()
                    } else {
                        "Plan Workflow State Machine".to_string()
                    }
                } else if state_context.contains("session") {
                    if state_context.contains("persist") {
                        "Session Persistence Manager".to_string()
                    } else {
                        "Session State Tracker".to_string()
                    }
                } else if state_context.contains("ui") || state_context.contains("view") {
                    "UI State Controller".to_string()
                } else if state_context.contains("modal") || state_context.contains("dialog") {
                    "Modal State Manager".to_string()
                } else {
                    format!("State Machine ({})", state_context)
                }
            },
            CodeCategory::EventHandling => {
                let event_types = self.extract_event_types();
                if event_types.iter().any(|s| s == "file" || s == "directory") {
                    "FileSystem Event Dispatcher".to_string()
                } else if event_types.iter().any(|s| s == "click" || s == "mouse" || s == "keyboard") {
                    "UI Interaction Handler".to_string()
                } else if event_types.iter().any(|s| s == "message" || s == "data") {
                    "Message Event Router".to_string()
                } else if event_types.iter().any(|s| s == "tool" || s == "function") {
                    "Tool Event Orchestrator".to_string()
                } else if event_types.iter().any(|s| s.contains("error") || s.contains("fail")) {
                    "Error Event Handler".to_string()
                } else if !event_types.is_empty() {
                    format!("Event Handler ({})", event_types.join(", "))
                } else {
                    "Generic Event Dispatcher".to_string()
                }
            },
            CodeCategory::DataValidation => {
                let validation_targets = self.extract_validation_targets();
                if validation_targets.iter().any(|s| s == "email") {
                    "Email Format Validator".to_string()
                } else if validation_targets.iter().any(|s| s == "url") {
                    "URL Format Validator".to_string()
                } else if validation_targets.iter().any(|s| s == "datetime") {
                    "DateTime Schema Validator".to_string()
                } else if validation_targets.iter().any(|s| s == "password") {
                    "Password Strength Validator".to_string()
                } else if validation_targets.iter().any(|s| s == "phone") {
                    "Phone Number Validator".to_string()
                } else if validation_targets.iter().any(|s| s == "schema") && self.code.contains("z.object") {
                    "Zod Schema Validator".to_string()
                } else if validation_targets.len() > 3 {
                    "Multi-Field Input Validator".to_string()
                } else if !validation_targets.is_empty() {
                    format!("{} Validator", validation_targets.join("_"))
                } else {
                    "Input Validator".to_string()
                }
            },
            CodeCategory::ErrorHandler => {
                if self.code.contains("retry") && self.code.contains("exponential") {
                    "Exponential Backoff Retry Handler".to_string()
                } else if self.code.contains("retry") {
                    "Error Recovery Handler".to_string()
                } else if self.code.contains("fallback") {
                    "Fallback Error Handler".to_string()
                } else if self.code.contains("API") || self.code.contains("api") {
                    "API Error Handler".to_string()
                } else if self.code.contains("network") {
                    "Network Error Handler".to_string()
                } else {
                    "Error Handler".to_string()
                }
            },
            CodeCategory::TelemetryRecorder => {
                if evidence.iter().any(|e| e.contains("tengu_")) {
                    "Telemetry Event Recorder".to_string()
                } else if self.code.contains("metric") && self.code.contains("aggregate") {
                    "Metrics Aggregator".to_string()
                } else if self.code.contains("track") && self.code.contains("user") {
                    "User Analytics Tracker".to_string()
                } else if self.code.contains("performance") {
                    "Performance Monitor".to_string()
                } else {
                    "Analytics Tracker".to_string()
                }
            },
            CodeCategory::PermissionControl => {
                if self.code.contains("plan") || self.code.contains("Plan") {
                    "Plan Mode Permission Enforcer".to_string()
                } else if self.code.contains("RBAC") || self.code.contains("role") {
                    "Role-Based Access Controller".to_string()
                } else if self.code.contains("ACL") {
                    "Access Control List Manager".to_string()
                } else if self.code.contains("file") && self.code.contains("path") {
                    "File Permission Checker".to_string()
                } else {
                    "Access Control Manager".to_string()
                }
            },
            CodeCategory::ProtocolHandler => {
                if self.code.contains("Claude-") {
                    "Claude Protocol Header Parser".to_string()
                } else if self.code.contains("HTTP") || self.code.contains("http") {
                    "HTTP Protocol Handler".to_string()
                } else if self.code.contains("WebSocket") || self.code.contains("ws://") {
                    "WebSocket Protocol Handler".to_string()
                } else if self.code.contains("gRPC") {
                    "gRPC Protocol Handler".to_string()
                } else {
                    "Protocol Handler".to_string()
                }
            },
            CodeCategory::WorkflowOrchestration => {
                if self.code.contains("step") && self.code.contains("pipeline") {
                    "Pipeline Orchestrator".to_string()
                } else if self.code.contains("saga") {
                    "Saga Workflow Coordinator".to_string()
                } else if self.code.contains("task") && self.code.contains("queue") {
                    "Task Queue Orchestrator".to_string()
                } else if self.code.contains("workflow") && self.code.contains("state") {
                    "Workflow State Coordinator".to_string()
                } else {
                    "Workflow Orchestrator".to_string()
                }
            },
            CodeCategory::ApiClient => {
                if self.code.contains("fetch") && self.code.contains("retry") {
                    "Resilient API Client".to_string()
                } else if self.code.contains("GraphQL") {
                    "GraphQL API Client".to_string()
                } else if self.code.contains("REST") || self.code.contains("RESTful") {
                    "REST API Client".to_string()
                } else if self.code.contains("axios") {
                    "Axios HTTP Client".to_string()
                } else {
                    "API Client".to_string()
                }
            },
            CodeCategory::MessageRouter => {
                if self.code.contains("RabbitMQ") || self.code.contains("AMQP") {
                    "RabbitMQ Message Router".to_string()
                } else if self.code.contains("Kafka") {
                    "Kafka Message Router".to_string()
                } else if self.code.contains("topic") && self.code.contains("subscribe") {
                    "PubSub Message Router".to_string()
                } else if self.code.contains("queue") {
                    "Queue Message Router".to_string()
                } else {
                    "Message Router".to_string()
                }
            },
            CodeCategory::ResourceManagement => {
                if self.code.contains("pool") && self.code.contains("connection") {
                    "Connection Pool Manager".to_string()
                } else if self.code.contains("memory") && self.code.contains("limit") {
                    "Memory Resource Manager".to_string()
                } else if self.code.contains("thread") || self.code.contains("worker") {
                    "Thread Pool Manager".to_string()
                } else if self.code.contains("file") && self.code.contains("descriptor") {
                    "File Descriptor Manager".to_string()
                } else {
                    "Resource Manager".to_string()
                }
            },
            CodeCategory::ConfigurationLoader => {
                if self.code.contains(".env") || self.code.contains("dotenv") {
                    "Environment Config Loader".to_string()
                } else if self.code.contains("yaml") || self.code.contains("YAML") {
                    "YAML Config Loader".to_string()
                } else if self.code.contains("json") && self.code.contains("parse") {
                    "JSON Config Loader".to_string()
                } else if self.code.contains("secret") || self.code.contains("vault") {
                    "Secret Config Loader".to_string()
                } else {
                    "Configuration Loader".to_string()
                }
            },
            CodeCategory::LoggingSystem => {
                if self.code.contains("winston") {
                    "Winston Logger".to_string()
                } else if self.code.contains("pino") {
                    "Pino Logger".to_string()
                } else if self.code.contains("structured") && self.code.contains("log") {
                    "Structured Logger".to_string()
                } else if self.code.contains("rotate") {
                    "Log Rotation Manager".to_string()
                } else {
                    "Logging System".to_string()
                }
            },
            CodeCategory::CacheManager => {
                if self.code.contains("Redis") {
                    "Redis Cache Manager".to_string()
                } else if self.code.contains("Memcached") {
                    "Memcached Manager".to_string()
                } else if self.code.contains("LRU") {
                    "LRU Cache Manager".to_string()
                } else if self.code.contains("TTL") || self.code.contains("expire") {
                    "TTL Cache Manager".to_string()
                } else {
                    "Cache Manager".to_string()
                }
            },
            CodeCategory::SyntaxHighlighter => {
                // Detect specific language
                if self.code.contains("XQuery") || self.code.contains("XPath") {
                    "XQuery Syntax Highlighter".to_string()
                } else if self.code.contains("TypeScript") || self.code.contains("JavaScript") {
                    "TypeScript/JavaScript Highlighter".to_string()
                } else if self.code.contains("Python") {
                    "Python Syntax Highlighter".to_string()
                } else if self.code.contains("SQL") {
                    "SQL Syntax Highlighter".to_string()
                } else if self.code.contains("XML") || self.code.contains("HTML") {
                    "XML/HTML Syntax Highlighter".to_string()
                } else if self.code.contains("CSS") {
                    "CSS Syntax Highlighter".to_string()
                } else {
                    "Syntax Highlighter".to_string()
                }
            },
            CodeCategory::LanguageParser => {
                if self.code.contains("AST") {
                    "AST Parser".to_string()
                } else if self.code.contains("lexer") {
                    "Lexical Analyzer".to_string()
                } else {
                    "Language Parser".to_string()
                }
            },
            CodeCategory::ReactComponent => "React Component".to_string(),
            CodeCategory::ReactHooks => "React Hooks".to_string(),
            CodeCategory::ReactEventHandler => "React Event Handler".to_string(),
            CodeCategory::ReactStateManager => "React State Manager".to_string(),
            CodeCategory::ReactContextProvider => "React Context Provider".to_string(),
            CodeCategory::DOMManipulator => "DOM Manipulator".to_string(),
            CodeCategory::DOMEventListener => "DOM Event Listener".to_string(),
            CodeCategory::VirtualDOMRenderer => "Virtual DOM Renderer".to_string(),
            CodeCategory::ProcessManager => "Process Manager".to_string(),
            CodeCategory::ChildProcessSpawner => "Child Process Spawner".to_string(),
            CodeCategory::StdioHandler => "Stdio Handler".to_string(),
            CodeCategory::EnvironmentVariableLoader => "Environment Variable Loader".to_string(),
            CodeCategory::ErrorRecovery => "Error Recovery".to_string(),
            CodeCategory::ExceptionLogger => "Exception Logger".to_string(),
            CodeCategory::PromiseRejectionHandler => "Promise Rejection Handler".to_string(),
            CodeCategory::AsyncOrchestrator => "Async Orchestrator".to_string(),
            CodeCategory::PromiseChain => "Promise Chain".to_string(),
            CodeCategory::AsyncIterator => "Async Iterator".to_string(),
            CodeCategory::FileSystemSync => "File System Sync".to_string(),
            CodeCategory::FileSystemAsync => "File System Async".to_string(),
            CodeCategory::PathResolver => "Path Resolver".to_string(),
            CodeCategory::DirectoryWatcher => "Directory Watcher".to_string(),
            CodeCategory::StringManipulator => "String Manipulator".to_string(),
            CodeCategory::RegexMatcher => "Regex Matcher".to_string(),
            CodeCategory::TextParser => "Text Parser".to_string(),
            CodeCategory::JSONParser => "JSON Parser".to_string(),
            CodeCategory::DataSerializer => "Data Serializer".to_string(),
            CodeCategory::URLEncoder => "URL Encoder".to_string(),
            CodeCategory::TimerScheduler => "Timer Scheduler".to_string(),
            CodeCategory::PerformanceMonitor => "Performance Monitor".to_string(),
            CodeCategory::AnimationFrameHandler => "Animation Frame Handler".to_string(),
            CodeCategory::OAuthFlowController => "OAuth Flow Controller".to_string(),
            CodeCategory::JWTTokenValidator => "JWT Token Validator".to_string(),
            CodeCategory::SessionCookieManager => "Session Cookie Manager".to_string(),
            CodeCategory::ReduxStore => "Redux Store".to_string(),
            CodeCategory::MobXStore => "MobX Store".to_string(),
            CodeCategory::ZustandStore => "Zustand Store".to_string(),
            CodeCategory::MessageQueueConsumer => "Message Queue Consumer".to_string(),
            CodeCategory::WebSocketEventHandler => "WebSocket Event Handler".to_string(),
            CodeCategory::FileSystemEventWatcher => "FileSystem Event Watcher".to_string(),
            CodeCategory::SchemaValidator => "Schema Validator".to_string(),
            CodeCategory::FormValidator => "Form Validator".to_string(),
            CodeCategory::PipelineExecutor => "Pipeline Executor".to_string(),
            CodeCategory::TaskQueueProcessor => "Task Queue Processor".to_string(),
            CodeCategory::HTTPClient => "HTTP Client".to_string(),
            CodeCategory::WebSocketClient => "WebSocket Client".to_string(),
            CodeCategory::ClaudeProtocolHandler => "Claude Protocol Handler".to_string(),
            CodeCategory::ClaudeTelemetryRecorder => "Claude Telemetry Recorder".to_string(),
            CodeCategory::ClaudeEnvironmentLoader => "Claude Environment Loader".to_string(),
            CodeCategory::SandboxManager => "Sandbox Manager".to_string(),
            CodeCategory::APIKeyVault => "API Key Vault".to_string(),
            CodeCategory::IDEConnector => "IDE Connector".to_string(),
            CodeCategory::CommandInjectionGuard => "Command Injection Guard".to_string(),
            CodeCategory::AgentTelemetryRecorder => "Agent Telemetry Recorder".to_string(),
            CodeCategory::APIMonitoringDashboard => "API Monitoring Dashboard".to_string(),
            CodeCategory::FeedbackCollectionSystem => "Feedback Collection System".to_string(),
            CodeCategory::SearchAnalytics => "Search Analytics".to_string(),
            CodeCategory::KeyboardShortcutManager => "Keyboard Shortcut Manager".to_string(),
            CodeCategory::CommandPaletteHandler => "Command Palette Handler".to_string(),
            CodeCategory::ModalDialogController => "Modal Dialog Controller".to_string(),
            CodeCategory::SentryIntegration => "Sentry Integration".to_string(),
            CodeCategory::ProxyManager => "Proxy Manager".to_string(),
            CodeCategory::TelemetryController => "Telemetry Controller".to_string(),
            CodeCategory::TokenBudgetManager => "Token Budget Manager".to_string(),
            CodeCategory::RetryPolicy => "Retry Policy".to_string(),
            CodeCategory::ToolConcurrencyLimiter => "Tool Concurrency Limiter".to_string(),
            CodeCategory::PromptSuggestionEngine => "Prompt Suggestion Engine".to_string(),
            CodeCategory::SDKCheckpointing => "SDK Checkpointing".to_string(),
            CodeCategory::BashSecurityMonitor => "Bash Security Monitor".to_string(),
            CodeCategory::GitHubIntegrationTracker => "GitHub Integration Tracker".to_string(),
            CodeCategory::PlanModeAnalytics => "Plan Mode Analytics".to_string(),
            CodeCategory::MCPOperationTracker => "MCP Operation Tracker".to_string(),
            CodeCategory::ToolUseMonitor => "Tool Use Monitor".to_string(),
            CodeCategory::VersionLockTracker => "Version Lock Tracker".to_string(),
            CodeCategory::OAuthFlowTracker => "OAuth Flow Tracker".to_string(),
            CodeCategory::TreeSitterLoader => "Tree-sitter Loader".to_string(),

            // Iteration 5-6 new categories
            CodeCategory::InputComponentLibrary => "Input Component Library".to_string(),
            CodeCategory::SelectComponentLibrary => "Select Component Library".to_string(),
            CodeCategory::FormComponentLibrary => "Form Component Library".to_string(),
            CodeCategory::TabNavigationSystem => "Tab Navigation System".to_string(),
            CodeCategory::ProgressIndicatorSystem => "Progress Indicator System".to_string(),
            CodeCategory::AlertNotificationSystem => "Alert Notification System".to_string(),
            CodeCategory::ButtonComponentLibrary => "Button Component Library".to_string(),
            CodeCategory::DialogComponentLibrary => "Dialog Component Library".to_string(),
            CodeCategory::MenuComponentLibrary => "Menu Component Library".to_string(),
            CodeCategory::ActionDispatcher => "Action Dispatcher".to_string(),
            CodeCategory::StateSelector => "State Selector".to_string(),
            CodeCategory::StoreManagerCore => "Store Manager".to_string(),
            CodeCategory::HTTPRequestManager => "HTTP Request Manager".to_string(),
            CodeCategory::HTTPResponseHandler => "HTTP Response Handler".to_string(),
            CodeCategory::EndpointRegistry => "Endpoint Registry".to_string(),
            CodeCategory::APIClientLibrary => "API Client Library".to_string(),
            CodeCategory::FetchAPIWrapper => "Fetch API Wrapper".to_string(),
            CodeCategory::DiffViewerComponent => "Diff Viewer Component".to_string(),
            CodeCategory::MergeConflictResolver => "Merge Conflict Resolver".to_string(),
            CodeCategory::CompactOperationManager => "Compact Operation Manager".to_string(),
            CodeCategory::TeleportNavigator => "Teleport Navigator".to_string(),
            CodeCategory::PlaneServiceCoordinator => "Plane Service Coordinator".to_string(),
            CodeCategory::CredentialsProviderSystem => "Credentials Provider System".to_string(),
            CodeCategory::TypeErrorHandler => "TypeError Handler".to_string(),
            CodeCategory::ParameterErrorHandler => "ParameterError Handler".to_string(),
            CodeCategory::ProviderErrorHandler => "ProviderError Handler".to_string(),
            CodeCategory::RangeErrorHandler => "RangeError Handler".to_string(),
            CodeCategory::ServiceExceptionHandler => "ServiceException Handler".to_string(),
            CodeCategory::AbortErrorHandler => "AbortError Handler".to_string(),
            CodeCategory::SyntaxErrorHandler => "SyntaxError Handler".to_string(),
            CodeCategory::TimeoutErrorHandler => "TimeoutError Handler".to_string(),
            CodeCategory::UnknownErrorHandler => "UnknownError Handler".to_string(),
            CodeCategory::QueryErrorHandler => "QueryError Handler".to_string(),
            CodeCategory::ReferenceErrorHandler => "ReferenceError Handler".to_string(),
            CodeCategory::ParseErrorHandler => "ParseError Handler".to_string(),
            CodeCategory::ResponseErrorHandler => "ResponseError Handler".to_string(),
            CodeCategory::RequestErrorHandler => "RequestError Handler".to_string(),
            CodeCategory::InternalErrorHandler => "InternalError Handler".to_string(),
            CodeCategory::TokenExceptionHandler => "TokenException Handler".to_string(),
            CodeCategory::ServerExceptionHandler => "ServerException Handler".to_string(),
            CodeCategory::AxiosErrorHandler => "AxiosError Handler".to_string(),
            CodeCategory::ParserErrorHandler => "ParserError Handler".to_string(),
            CodeCategory::AuthErrorHandler => "AuthError Handler".to_string(),

            // Iteration 13: Content-based categories
            CodeCategory::EllipticCurveCrypto => "Elliptic Curve Cryptography".to_string(),
            CodeCategory::NodeErrorFactory => "Node.js Error Factory".to_string(),
            CodeCategory::BitwiseCryptoOps => "Bitwise Crypto Operations".to_string(),
            CodeCategory::JavaScriptSyntaxHighlighter => "JavaScript Syntax Highlighter".to_string(),
            CodeCategory::ReactDevToolsProfiler => "React DevTools Profiler".to_string(),
            CodeCategory::SyncFileIO => "Synchronous File I/O".to_string(),
            CodeCategory::ImageProcessor => "Image Processor".to_string(),
            CodeCategory::RegexEngine => "Regex Engine".to_string(),
            CodeCategory::TimestampManager => "Timestamp Manager".to_string(),
            CodeCategory::APIErrorHandler => "API Error Handler".to_string(),
            CodeCategory::ErrorRecoverySystem => "Error Recovery System".to_string(),
            CodeCategory::FallbackErrorHandler => "Fallback Error Handler".to_string(),
            CodeCategory::PromiseErrorHandler => "Promise Error Handler".to_string(),

            // Iteration 14: Additional obfuscated utilities
            CodeCategory::DateFnsLibrary => "Date-fns Library".to_string(),
            CodeCategory::DebounceThrottle => "Debounce/Throttle".to_string(),
            CodeCategory::JSONTokenizer => "JSON Tokenizer".to_string(),
            CodeCategory::StartupProfiler => "Startup Profiler".to_string(),
            CodeCategory::ObjectInspector => "Object Inspector".to_string(),
            CodeCategory::ElmSyntaxHighlighter => "Elm Syntax Highlighter".to_string(),
            CodeCategory::LodashTypeChecker => "Lodash Type Checker".to_string(),

            // Iteration 20: Deep obfuscation categories
            CodeCategory::RxJSOperators => "RxJS Operators".to_string(),
            CodeCategory::OpenTelemetryEncoding => "OpenTelemetry Encoding".to_string(),
            CodeCategory::ZlibCompression => "Zlib Compression".to_string(),
            CodeCategory::InstallationDetection => "Installation Detection".to_string(),
            CodeCategory::AnthropicAPIClient => "Anthropic API Client".to_string(),
            CodeCategory::LodashCoreLibrary => "Lodash Core Library".to_string(),

            // Iteration 24: Crypto library wrappers
            CodeCategory::CryptoLibraryWrappers => "Crypto Library Wrappers".to_string(),

            CodeCategory::Unknown => "Unknown Module".to_string(),
        }
    }

    /// Get semantic filename based on behavioral analysis
    pub fn get_semantic_filename(&self) -> Option<String> {
        let profile = self.analyze();

        if profile.confidence < 0.5 {
            return None;
        }

        let path = match profile.category {
            CodeCategory::AWSCredentialProvider => {
                format!("src/auth/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ErrorHandler => {
                format!("src/errors/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::DataValidation => {
                format!("src/validation/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::StateManagement => {
                format!("src/state/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::EventHandling => {
                format!("src/events/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TelemetryRecorder => {
                format!("src/telemetry/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::PermissionControl => {
                format!("src/permissions/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ProtocolHandler => {
                format!("src/protocol/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SyntaxHighlighter => {
                format!("src/languages/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::LanguageParser => {
                format!("src/parsers/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::WorkflowOrchestration => {
                format!("src/workflows/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ApiClient => {
                format!("src/api/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::MessageRouter => {
                format!("src/messaging/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ResourceManagement => {
                format!("src/resources/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ConfigurationLoader => {
                format!("src/config/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::LoggingSystem => {
                format!("src/logging/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::CacheManager => {
                format!("src/cache/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SandboxManager => {
                format!("src/sandbox/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::APIKeyVault => {
                format!("src/security/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::IDEConnector => {
                format!("src/integrations/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::CommandInjectionGuard => {
                format!("src/security/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AgentTelemetryRecorder => {
                format!("src/telemetry/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::APIMonitoringDashboard => {
                format!("src/monitoring/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::FeedbackCollectionSystem => {
                format!("src/feedback/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SearchAnalytics => {
                format!("src/analytics/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::KeyboardShortcutManager => {
                format!("src/keyboard/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::CommandPaletteHandler => {
                format!("src/commands/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ModalDialogController => {
                format!("src/ui/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SentryIntegration => {
                format!("src/monitoring/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::BashSecurityMonitor => {
                format!("src/security/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ProxyManager => {
                format!("src/network/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TelemetryController => {
                format!("src/telemetry/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TokenBudgetManager => {
                format!("src/resources/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::RetryPolicy => {
                format!("src/resilience/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ToolConcurrencyLimiter => {
                format!("src/concurrency/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::PromptSuggestionEngine => {
                format!("src/ai/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SDKCheckpointing => {
                format!("src/state/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::GitHubIntegrationTracker => {
                format!("src/integrations/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::PlanModeAnalytics => {
                format!("src/analytics/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::MCPOperationTracker => {
                format!("src/mcp/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ToolUseMonitor => {
                format!("src/monitoring/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::VersionLockTracker => {
                format!("src/versioning/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::OAuthFlowTracker => {
                format!("src/auth/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TreeSitterLoader => {
                format!("src/parsers/{}.ts.AI", self.slugify(&profile.purpose))
            },

            // Iteration 5-6 new categories
            CodeCategory::InputComponentLibrary => {
                format!("src/components/input/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SelectComponentLibrary => {
                format!("src/components/select/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::FormComponentLibrary => {
                format!("src/components/form/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TabNavigationSystem => {
                format!("src/components/tabs/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ProgressIndicatorSystem => {
                format!("src/components/progress/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AlertNotificationSystem => {
                format!("src/components/alert/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ButtonComponentLibrary => {
                format!("src/components/button/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::DialogComponentLibrary => {
                format!("src/components/dialog/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::MenuComponentLibrary => {
                format!("src/components/menu/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ActionDispatcher => {
                format!("src/state/actions/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::StateSelector => {
                format!("src/state/selectors/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::StoreManagerCore => {
                format!("src/state/store/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::HTTPRequestManager => {
                format!("src/network/http/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::HTTPResponseHandler => {
                format!("src/network/response/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::EndpointRegistry => {
                format!("src/network/endpoints/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::APIClientLibrary => {
                format!("src/network/api/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::FetchAPIWrapper => {
                format!("src/network/fetch/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::DiffViewerComponent => {
                format!("src/editor/diff/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::MergeConflictResolver => {
                format!("src/editor/merge/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::CompactOperationManager => {
                format!("src/editor/compact/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TeleportNavigator => {
                format!("src/editor/teleport/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::PlaneServiceCoordinator => {
                format!("src/services/plane/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::CredentialsProviderSystem => {
                format!("src/services/credentials/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TypeErrorHandler => {
                format!("src/errors/type/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ParameterErrorHandler => {
                format!("src/errors/parameter/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ProviderErrorHandler => {
                format!("src/errors/provider/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::RangeErrorHandler => {
                format!("src/errors/range/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ServiceExceptionHandler => {
                format!("src/errors/service/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AbortErrorHandler => {
                format!("src/errors/abort/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SyntaxErrorHandler => {
                format!("src/errors/syntax/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TimeoutErrorHandler => {
                format!("src/errors/timeout/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::UnknownErrorHandler => {
                format!("src/errors/unknown/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::QueryErrorHandler => {
                format!("src/errors/query/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ReferenceErrorHandler => {
                format!("src/errors/reference/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ParseErrorHandler => {
                format!("src/errors/parse/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ResponseErrorHandler => {
                format!("src/errors/response/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::RequestErrorHandler => {
                format!("src/errors/request/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::InternalErrorHandler => {
                format!("src/errors/internal/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TokenExceptionHandler => {
                format!("src/errors/token/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ServerExceptionHandler => {
                format!("src/errors/server/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AxiosErrorHandler => {
                format!("src/errors/axios/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ParserErrorHandler => {
                format!("src/errors/parser/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AuthErrorHandler => {
                format!("src/errors/auth/{}.ts.AI", self.slugify(&profile.purpose))
            },

            // Iteration 13: Content-based categories (defeats filename obfuscation)
            CodeCategory::EllipticCurveCrypto => {
                format!("src/crypto/elliptic_curve/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::NodeErrorFactory => {
                format!("src/errors/node/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::BitwiseCryptoOps => {
                format!("src/crypto/bitwise/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::JavaScriptSyntaxHighlighter => {
                format!("src/languages/javascript/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ReactDevToolsProfiler => {
                format!("src/devtools/react/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::SyncFileIO => {
                format!("src/fs/sync/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ImageProcessor => {
                format!("src/media/images/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::RegexEngine => {
                format!("src/validation/regex/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::TimestampManager => {
                format!("src/time/timestamps/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::APIErrorHandler => {
                format!("src/errors/api/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ErrorRecoverySystem => {
                format!("src/errors/recovery/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::FallbackErrorHandler => {
                format!("src/errors/fallback/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::PromiseErrorHandler => {
                format!("src/async/errors/{}.ts.AI", self.slugify(&profile.purpose))
            },

            // Iteration 14: Additional obfuscated utilities
            CodeCategory::DateFnsLibrary => {
                format!("src/time/datefns/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::DebounceThrottle => {
                format!("src/utils/timers/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::JSONTokenizer => {
                format!("src/parsers/json/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::StartupProfiler => {
                format!("src/profiling/startup/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ObjectInspector => {
                format!("src/introspection/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ElmSyntaxHighlighter => {
                format!("src/languages/elm/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::LodashTypeChecker => {
                format!("src/utils/lodash/{}.ts.AI", self.slugify(&profile.purpose))
            },

            // Iteration 20: Deep obfuscation path mappings
            CodeCategory::RxJSOperators => {
                format!("src/reactive/rxjs/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::OpenTelemetryEncoding => {
                format!("src/telemetry/opentelemetry/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::ZlibCompression => {
                format!("src/compression/zlib/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::InstallationDetection => {
                format!("src/installer/detection/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::AnthropicAPIClient => {
                format!("src/api/anthropic/{}.ts.AI", self.slugify(&profile.purpose))
            },
            CodeCategory::LodashCoreLibrary => {
                format!("src/libraries/lodash/{}.ts.AI", self.slugify(&profile.purpose))
            },

            // Iteration 24: Crypto library wrappers
            CodeCategory::CryptoLibraryWrappers => {
                format!("src/crypto/wrappers/{}.ts.AI", self.slugify(&profile.purpose))
            },

            _ => return None,
        };

        Some(path)
    }

    fn slugify(&self, s: &str) -> String {
        s.to_lowercase()
            .replace(" ", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    /// Extract specific state names from state machine code
    fn extract_state_context(&self) -> String {
        let state_pattern = Regex::new(r#"(state|mode|status|phase)\s*===?\s*["']([^"']+)["']"#).unwrap();
        let states: Vec<String> = state_pattern.captures_iter(&self.code)
            .filter_map(|m| m.get(2).map(|s| s.as_str().to_string()))
            .take(3)
            .collect();

        if states.is_empty() {
            "generic".to_string()
        } else {
            states.join("_")
        }
    }

    /// Extract event type names from switch/case statements
    fn extract_event_types(&self) -> Vec<String> {
        let case_pattern = Regex::new(r#"case\s+["']([a-z_]+)["']:"#).unwrap();
        case_pattern.captures_iter(&self.code)
            .filter_map(|m| m.get(1).map(|s| s.as_str().to_string()))
            .filter(|s| !s.contains("invalid") && !s.contains("error"))
            .take(3)
            .collect()
    }

    /// Extract validation field names to understand what's being validated
    fn extract_validation_targets(&self) -> Vec<String> {
        let mut targets = Vec::new();

        // Check for specific field validations
        if self.code.contains("email") {
            targets.push("email".to_string());
        }
        if self.code.contains("url") || self.code.contains("URL") {
            targets.push("url".to_string());
        }
        if self.code.contains("datetime") || self.code.contains("date") {
            targets.push("datetime".to_string());
        }
        if self.code.contains("password") {
            targets.push("password".to_string());
        }
        if self.code.contains("phone") {
            targets.push("phone".to_string());
        }

        if targets.is_empty() {
            targets.push("input".to_string());
        }

        targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_detection() {
        let code = r#"
        function controller(state) {
            if (state.mode === "planning") return planMode();
            if (state.mode === "executing") return executeMode();
            if (state.mode === "reviewing") return reviewMode();
        }
        "#.to_string();

        let analyzer = BehaviorAnalyzer::new(code);
        let profile = analyzer.analyze();

        assert_eq!(profile.category, CodeCategory::StateManagement);
        assert!(profile.confidence > 0.5);
    }

    #[test]
    fn test_validation_detection() {
        let code = r#"
        const schema = z.object({
            email: z.string().email(),
            age: z.number().min(0).max(150),
        });
        // Returns: invalid_type, invalid_format, too_small, too_big
        "#.to_string();

        let analyzer = BehaviorAnalyzer::new(code);
        let profile = analyzer.analyze();

        assert_eq!(profile.category, CodeCategory::DataValidation);
    }
}
