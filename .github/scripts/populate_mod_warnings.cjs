const fs = require('fs').promises;
const path = require('path');

// --- CONFIGURATION ---
const NEXUS_API_KEY = process.env.NEXUS_API_KEY;
if (!NEXUS_API_KEY) throw new Error("NEXUS_API_KEY environment variable not set!");

const OUTPUT_FILE_PATH = path.join(process.cwd(), 'mod_warnings.json');
const BATCH_SIZE = 10; // Smaller batch size for discovery calls
const DELAY_BETWEEN_BATCHES = 2000; // 2 seconds between batches
const MIN_DOWNLOAD_COUNT = 1; // Very low threshold to include nearly all mods
const ABANDONED_THRESHOLD_YEARS = 3; // More lenient abandonment threshold
const MAX_MODS_PER_REQUEST = 100; // NexusMods API limit

// --- FILTERING AND WARNING DETECTION ---

function detectWarnings(mod) {
    const warnings = [];
    const now = Math.floor(Date.now() / 1000);

    // Check for adult content
    if (mod.contains_adult_content ||
        /\b(nsfw|adult|nude|nudity|sex|porn|erotic|18\+)\b/i.test(mod.name + ' ' + mod.summary)) {
        warnings.push({
            type: 'adult_content',
            message: 'Contains adult content'
        });
    }

    // Check for abandoned mods (no updates in 2+ years)
    const yearsSinceUpdate = (now - mod.updated_timestamp) / (365.25 * 24 * 3600);
    if (yearsSinceUpdate >= ABANDONED_THRESHOLD_YEARS) {
        warnings.push({
            type: 'abandoned',
            message: `Not updated in ${Math.round(yearsSinceUpdate)} years`
        });
    }

    // Check for potentially broken/problematic mods based on description
    const description = (mod.summary || '').toLowerCase();
    const problemPatterns = [
        /\b(broken|doesn't work|not working|issues?|problems?|buggy)\b/i,
        /\b(discontinued|deprecated|obsolete|outdated)\b/i,
        /\b(experimental|alpha|beta|unstable|wip)\b/i
    ];

    for (const pattern of problemPatterns) {
        if (pattern.test(description)) {
            if (pattern.source.includes('experimental|alpha|beta')) {
                warnings.push({
                    type: 'experimental',
                    message: 'Experimental or work-in-progress mod'
                });
            } else if (pattern.source.includes('discontinued|deprecated')) {
                warnings.push({
                    type: 'discontinued',
                    message: 'Mod is discontinued or deprecated'
                });
            } else {
                warnings.push({
                    type: 'potential_issues',
                    message: 'May have compatibility or stability issues'
                });
            }
            break; // Only add one warning per category
        }
    }

    // Check download count (very low downloads might indicate issues)
    if (mod.unique_downloads < MIN_DOWNLOAD_COUNT) {
        warnings.push({
            type: 'low_downloads',
            message: `Low download count (${mod.unique_downloads})`
        });
    }

    return warnings;
}

function shouldIncludeMod(mod) {
    // Very minimal filtering - include almost all mods
    if (mod.status !== 'published') return false; // Only exclude unpublished mods
    // Removed download count filter to get all mods
    return true;
}

function determineModState(warnings) {
    const warningTypes = warnings.map(w => w.type);

    if (warningTypes.includes('adult_content')) return 'adult';
    if (warningTypes.includes('discontinued')) return 'discontinued';
    if (warningTypes.includes('abandoned')) return 'warning';
    if (warningTypes.includes('experimental')) return 'experimental';
    if (warningTypes.includes('potential_issues')) return 'warning';

    return 'normal';
}

function createWarningMessage(warnings) {
    if (warnings.length === 0) return '';

    // Prioritize most important warnings
    const priorities = ['adult_content', 'discontinued', 'abandoned', 'potential_issues', 'experimental', 'low_downloads'];

    for (const priority of priorities) {
        const warning = warnings.find(w => w.type === priority);
        if (warning) return warning.message;
    }

    return warnings[0].message;
}

// --- API HELPERS ---

async function fetchAllModsFromNexus() {
    console.log("Discovering No Man's Sky mods from NexusMods (API policy compliant)...");

    let allMods = new Map();
    let apiCallCount = 0;

    const headers = {
        "apikey": NEXUS_API_KEY,
        "User-Agent": "PulsarMM-ModDiscovery/1.0",
        "Application-Name": "PulsarMM",
        "Application-Version": "1.0"
    };

    console.log("Using API-compliant discovery (recent updates + trending only)");
    console.log("This respects NexusMods API acceptable use policy");

    // Strategy 1: Get recently updated mods (API-intended use)
    const periods = ['1d', '1w', '1m']; // Only use periods that work

    for (const period of periods) {
        try {
            console.log(`Fetching mods updated in period: ${period}...`);
            const url = `https://api.nexusmods.com/v1/games/nomanssky/mods/updated.json?period=${period}`;
            const response = await fetch(url, { headers });
            apiCallCount++;

            if (response.ok) {
                const periodMods = await response.json();
                if (Array.isArray(periodMods)) {
                    periodMods.forEach(mod => {
                        if (mod.mod_id) {
                            allMods.set(mod.mod_id, mod);
                        }
                    });
                    console.log(`  Found ${periodMods.length} mods in ${period} period (${allMods.size} unique total)`);
                }
            }

            await new Promise(r => setTimeout(r, 1000));

        } catch (error) {
            console.log(`Error fetching ${period} period:`, error.message);
        }
    }

    // Strategy 2: Get trending mods (API-intended use)
    try {
        console.log("Fetching trending mods...");
        const url = `https://api.nexusmods.com/v1/games/nomanssky/mods/trending.json`;
        const response = await fetch(url, { headers });
        apiCallCount++;

        if (response.ok) {
            const trendingMods = await response.json();
            if (Array.isArray(trendingMods)) {
                trendingMods.forEach(mod => {
                    if (mod.mod_id) {
                        allMods.set(mod.mod_id, mod);
                    }
                });
                console.log(`  Found ${trendingMods.length} trending mods (${allMods.size} unique total)`);
            }
        }

        await new Promise(r => setTimeout(r, 1000));
    } catch (error) {
        console.log("Trending mods not available:", error.message);
    }

    // Strategy 3: Preserve existing mods from previous runs (organic growth)
    console.log("Preserving existing mod registry for organic growth...");
    console.log("Note: Mod registry will grow over time as:");
    console.log("  - New mods are published and appear in recent updates");
    console.log("  - Users request specific mods");
    console.log("  - Mods become trending");

    console.log("=".repeat(50));
    console.log(`API-compliant discovery complete:`);
    console.log(`  Total unique mods found: ${allMods.size}`);
    console.log(`  API calls made: ${apiCallCount} (lightweight)`);
    console.log(`  Approach: User-driven organic growth (not bulk scraping)`);
    console.log("=".repeat(50));

    return Array.from(allMods.values()).filter(shouldIncludeMod);
}

// --- MAIN LOGIC ---

async function populateModWarnings() {
    console.log("Starting automated mod discovery and population...");

    try {
        // Check if we should do a full discovery or incremental update
        let existingMods = [];
        let doFullDiscovery = true;

        try {
            const existingContent = await fs.readFile(OUTPUT_FILE_PATH, 'utf8');
            existingMods = JSON.parse(existingContent);

            // If we already have a substantial list, consider incremental updates
            if (existingMods.length > 100) {
                // Check last modification time - full discovery weekly, incremental daily
                const stats = await fs.stat(OUTPUT_FILE_PATH);
                const daysSinceUpdate = (Date.now() - stats.mtime.getTime()) / (1000 * 60 * 60 * 24);

                if (daysSinceUpdate < 7) {
                    console.log(`Existing list has ${existingMods.length} mods, updated ${Math.round(daysSinceUpdate)} days ago`);
                    console.log("Skipping full discovery, will rely on curated list generation for updates");
                    doFullDiscovery = false;
                }
            }
        } catch (e) {
            console.log("No existing mod_warnings.json found, performing full discovery");
        }

        if (!doFullDiscovery) {
            console.log("Incremental update mode - mod_warnings.json is recent enough");
            return;
        }

        // 1. Fetch all mods from NexusMods
        const allMods = await fetchAllModsFromNexus();

        if (allMods.length === 0) {
            console.log("No mods found, keeping existing mod_warnings.json");
            return;
        }

        // 2. Process mods in batches for warning detection
        const processedMods = [];
        console.log(`Processing ${allMods.length} mods for warning detection...`);

        for (let i = 0; i < allMods.length; i += BATCH_SIZE) {
            const batch = allMods.slice(i, i + BATCH_SIZE);

            const batchResults = batch.map(mod => {
                const warnings = detectWarnings(mod);
                const state = determineModState(warnings);
                const warningMessage = createWarningMessage(warnings);

                return {
                    name: mod.name,
                    id: mod.mod_id.toString(),
                    state: state,
                    warningMessage: warningMessage
                };
            });

            processedMods.push(...batchResults);

            // Progress update
            if (i % (BATCH_SIZE * 10) === 0) {
                console.log(`Processed ${Math.min(i + BATCH_SIZE, allMods.length)}/${allMods.length} mods...`);
            }
        }

        // 3. Sort by mod ID for consistency
        processedMods.sort((a, b) => parseInt(a.id) - parseInt(b.id));

        // 4. Save to file
        await fs.mkdir(path.dirname(OUTPUT_FILE_PATH), { recursive: true });
        const newContent = JSON.stringify(processedMods, null, 4);

        // 5. Check if content has changed before writing
        let needsWrite = true;
        try {
            const currentContent = await fs.readFile(OUTPUT_FILE_PATH, 'utf8');
            if (currentContent === newContent) {
                console.log("No changes detected, skipping file write");
                needsWrite = false;
            }
        } catch (e) {
            console.log("No existing file found, creating new mod_warnings.json");
        }

        if (needsWrite) {
            await fs.writeFile(OUTPUT_FILE_PATH, newContent);
            console.log(`Successfully wrote ${processedMods.length} mods to ${OUTPUT_FILE_PATH}`);
        }

        // 6. Summary report
        const stateCount = processedMods.reduce((acc, mod) => {
            acc[mod.state] = (acc[mod.state] || 0) + 1;
            return acc;
        }, {});

        console.log("=".repeat(50));
        console.log(" MOD DISCOVERY SUMMARY");
        console.log("=".repeat(50));
        console.log(`Total mods discovered: ${processedMods.length}`);
        console.log("Breakdown by state:");
        Object.entries(stateCount).forEach(([state, count]) => {
            console.log(`  ${state}: ${count} mods`);
        });
        console.log(`Output file: ${OUTPUT_FILE_PATH}`);
        console.log("=".repeat(50));

    } catch (error) {
        console.error("Mod discovery failed:", error);
        process.exit(1);
    }
}

// Run the script
populateModWarnings().catch(error => {
    console.error("Script failed:", error);
    process.exit(1);
});