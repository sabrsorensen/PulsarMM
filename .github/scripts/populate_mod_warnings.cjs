const fs = require('fs').promises;
const path = require('path');

// --- CONFIGURATION ---
const NEXUS_API_KEY = process.env.NEXUS_API_KEY;
if (!NEXUS_API_KEY) throw new Error("NEXUS_API_KEY environment variable not set!");

const OUTPUT_FILE_PATH = path.join(process.cwd(), 'mod_warnings.json');
const BATCH_SIZE = 10; // Smaller batch size for discovery calls
const DELAY_BETWEEN_BATCHES = 2000; // 2 seconds between batches
const MIN_DOWNLOAD_COUNT = 10; // Minimum downloads to include mod
const ABANDONED_THRESHOLD_YEARS = 2; // Years without update = abandoned
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
    // Filter criteria
    if (mod.status !== 'published') return false;
    if (mod.unique_downloads < MIN_DOWNLOAD_COUNT) return false;
    if (mod.contains_adult_content && mod.unique_downloads < 100) return false; // Allow popular adult mods

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
    console.log("Fetching all No Man's Sky mods from NexusMods...");

    // Try to get all mods in a single request first
    const url = `https://api.nexusmods.com/v1/games/nomanssky/mods.json`;
    const headers = {
        "apikey": NEXUS_API_KEY,
        "User-Agent": "PulsarMM-ModDiscovery/1.0"
    };

    try {
        console.log("Attempting to fetch all mods in single request...");
        const response = await fetch(url, { headers });

        if (response.status === 429) {
            console.log("Rate limited, waiting 60 seconds...");
            await new Promise(r => setTimeout(r, 60000));
            return fetchAllModsFromNexus(); // Retry
        }

        if (!response.ok) {
            throw new Error(`API error ${response.status}: ${response.statusText}`);
        }

        const allMods = await response.json();

        if (!Array.isArray(allMods)) {
            throw new Error("Unexpected API response format");
        }

        console.log(`Discovery complete: Found ${allMods.length} total mods`);

        // Filter mods based on our criteria
        const filteredMods = allMods.filter(shouldIncludeMod);
        console.log(`After filtering: ${filteredMods.length} mods included`);

        console.log(`API calls made: 1`);
        return filteredMods;

    } catch (error) {
        console.error("Error fetching mods:", error.message);
        throw error;
    }
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