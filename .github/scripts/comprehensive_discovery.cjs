const fs = require('fs').promises;

// Comprehensive mod discovery using systematic ID scanning
const NEXUS_API_KEY = process.env.NEXUS_API_KEY;
if (!NEXUS_API_KEY) throw new Error("NEXUS_API_KEY environment variable not set!");

const headers = {
    "apikey": NEXUS_API_KEY,
    "User-Agent": "PulsarMM-ModDiscovery/1.0"
};

async function testPeriodValues() {
    console.log("\n🔍 Testing different period values...");

    const periods = [
        '1d', '3d', '7d', '1w', '2w', '1m', '2m', '3m', '6m', '1y', '2y', 'all'
    ];

    const workingPeriods = [];

    for (const period of periods) {
        try {
            const url = `https://api.nexusmods.com/v1/games/nomanssky/mods/updated.json?period=${period}`;
            console.log(`Testing period: ${period}`);

            const response = await fetch(url, { headers });

            if (response.ok) {
                const data = await response.json();
                if (Array.isArray(data)) {
                    console.log(`  ✅ ${period}: ${data.length} mods`);
                    workingPeriods.push({period, count: data.length, data});
                }
            } else {
                console.log(`  ❌ ${period}: ${response.status} ${response.statusText}`);
            }

            await new Promise(r => setTimeout(r, 1000));

        } catch (error) {
            console.log(`  ❌ ${period}: ${error.message}`);
        }
    }

    return workingPeriods;
}

async function discoverModIdRange() {
    console.log("\n🔍 Discovering mod ID range...");

    // Get mod IDs from known working endpoints
    let knownIds = new Set();

    try {
        // Get recent mods
        const recentResponse = await fetch(`https://api.nexusmods.com/v1/games/nomanssky/mods/updated.json?period=1m`, { headers });
        if (recentResponse.ok) {
            const recentMods = await recentResponse.json();
            recentMods.forEach(mod => knownIds.add(mod.mod_id));
        }

        await new Promise(r => setTimeout(r, 1000));

        // Get trending mods
        const trendingResponse = await fetch(`https://api.nexusmods.com/v1/games/nomanssky/mods/trending.json`, { headers });
        if (trendingResponse.ok) {
            const trendingMods = await trendingResponse.json();
            trendingMods.forEach(mod => knownIds.add(mod.mod_id));
        }

    } catch (error) {
        console.log(`Error getting known IDs: ${error.message}`);
    }

    const knownIdArray = Array.from(knownIds).sort((a, b) => a - b);
    const minId = Math.min(...knownIdArray);
    const maxId = Math.max(...knownIdArray);

    console.log(`Known ID range: ${minId} to ${maxId} (${knownIdArray.length} known IDs)`);
    console.log(`Estimated gap: ${maxId - minId - knownIdArray.length} potential missing mods`);

    return { minId, maxId, knownIds: knownIdArray };
}

async function systematicModIdScan(minId, maxId, knownIds, maxScans = 100) {
    console.log(`\n🔍 Systematic mod ID scanning from ${minId} to ${maxId}...`);
    console.log(`Will sample up to ${maxScans} IDs to respect rate limits\n`);

    const foundMods = [];
    let apiCalls = 0;
    const step = Math.max(1, Math.floor((maxId - minId) / maxScans));

    for (let id = minId; id <= maxId && apiCalls < maxScans; id += step) {
        if (knownIds.includes(id)) continue; // Skip IDs we already have

        try {
            const url = `https://api.nexusmods.com/v1/games/nomanssky/mods/${id}.json`;
            const response = await fetch(url, { headers });
            apiCalls++;

            if (response.ok) {
                const mod = await response.json();
                if (mod && mod.mod_id && mod.status === 'published') {
                    foundMods.push(mod);
                    console.log(`  Found mod ${id}: "${mod.name}" (downloads: ${mod.unique_downloads || 'unknown'})`);
                }
            }

            // Show progress every 10 calls
            if (apiCalls % 10 === 0) {
                console.log(`    Progress: ${apiCalls}/${maxScans} API calls, ${foundMods.length} new mods found`);
            }

            await new Promise(r => setTimeout(r, 200)); // Rate limiting

        } catch (error) {
            console.log(`  Error checking ID ${id}: ${error.message}`);
        }
    }

    console.log(`\nScan complete: ${foundMods.length} new mods discovered with ${apiCalls} API calls`);
    return foundMods;
}

async function comprehensiveModDiscovery() {
    console.log("🚀 Starting comprehensive mod discovery for all 2713 No Man's Sky mods...\n");

    try {
        // Step 1: Test different period values
        const workingPeriods = await testPeriodValues();

        // Step 2: Discover mod ID range
        const { minId, maxId, knownIds } = await discoverModIdRange();

        // Step 3: Systematic ID scanning
        const scannedMods = await systematicModIdScan(minId, maxId, knownIds, 200); // Increase scan count

        // Step 4: Combine all discoveries
        let allMods = new Map();

        // Add mods from working periods
        workingPeriods.forEach(period => {
            period.data.forEach(mod => {
                if (mod.mod_id) allMods.set(mod.mod_id, mod);
            });
        });

        // Add scanned mods
        scannedMods.forEach(mod => {
            allMods.set(mod.mod_id, mod);
        });

        console.log("\n" + "=".repeat(60));
        console.log(" COMPREHENSIVE DISCOVERY RESULTS");
        console.log("=".repeat(60));
        console.log(`Working time periods: ${workingPeriods.length}`);
        workingPeriods.forEach(p => console.log(`  ${p.period}: ${p.count} mods`));
        console.log(`ID scanning discovered: ${scannedMods.length} additional mods`);
        console.log(`Total unique mods found: ${allMods.size}`);
        console.log(`Target: 2713 total mods`);
        console.log(`Coverage: ${(allMods.size / 2713 * 100).toFixed(1)}%`);

        if (allMods.size < 2000) {
            console.log(`\n⚠️  Still missing ~${2713 - allMods.size} mods!`);
            console.log("Recommendations:");
            console.log("- Increase systematic scanning range");
            console.log("- Try different scanning strategies");
            console.log("- Check if mods are in unpublished/hidden state");
            console.log("- Consider that 2713 might include deleted/hidden mods");
        }

        console.log("=".repeat(60));

        return Array.from(allMods.values());

    } catch (error) {
        console.error("Comprehensive discovery failed:", error);
        throw error;
    }
}

// Run comprehensive discovery
comprehensiveModDiscovery().catch(error => {
    console.error("Discovery failed:", error);
    process.exit(1);
});