const fs = require('fs').promises;

// Test different NexusMods API endpoints to find how to get all mods
const NEXUS_API_KEY = process.env.NEXUS_API_KEY;
if (!NEXUS_API_KEY) throw new Error("NEXUS_API_KEY environment variable not set!");

const headers = {
    "apikey": NEXUS_API_KEY,
    "User-Agent": "PulsarMM-ModDiscovery/1.0"
};

async function testEndpoint(url, description) {
    console.log(`\nTesting: ${description}`);
    console.log(`URL: ${url}`);

    try {
        const response = await fetch(url, { headers });
        console.log(`Status: ${response.status} ${response.statusText}`);

        if (response.ok) {
            const data = await response.json();

            if (Array.isArray(data)) {
                console.log(`✅ Array with ${data.length} items`);
                if (data.length > 0) {
                    console.log(`First item keys: ${Object.keys(data[0]).join(', ')}`);
                }
            } else if (typeof data === 'object') {
                console.log(`✅ Object with keys: ${Object.keys(data).join(', ')}`);

                // Check if it has pagination info or mod arrays
                if (data.mods) console.log(`  mods array: ${data.mods.length} items`);
                if (data.total) console.log(`  total: ${data.total}`);
                if (data.per_page) console.log(`  per_page: ${data.per_page}`);
                if (data.page) console.log(`  page: ${data.page}`);
            } else {
                console.log(`✅ ${typeof data}: ${data}`);
            }
        } else {
            console.log(`❌ Failed`);
        }

        await new Promise(r => setTimeout(r, 1000)); // Rate limiting delay

    } catch (error) {
        console.log(`❌ Error: ${error.message}`);
    }
}

async function discoverEndpoints() {
    console.log("🔍 Discovering NexusMods API endpoints for all No Man's Sky mods...\n");

    const baseUrl = "https://api.nexusmods.com/v1/games/nomanssky";

    // Test various endpoint patterns
    const endpointsToTest = [
        // Basic mod endpoints
        [`${baseUrl}/mods`, "Base mods endpoint"],
        [`${baseUrl}/mods/`, "Base mods endpoint with trailing slash"],
        [`${baseUrl}/mods.json`, "Base mods endpoint with .json"],

        // Pagination attempts
        [`${baseUrl}/mods?page=1`, "Mods with page parameter"],
        [`${baseUrl}/mods?page=1&per_page=100`, "Mods with pagination"],
        [`${baseUrl}/mods?limit=100`, "Mods with limit parameter"],
        [`${baseUrl}/mods?offset=0&limit=100`, "Mods with offset/limit"],

        // Category-based (common categories)
        [`${baseUrl}/mods?category=1`, "Mods category 1"],
        [`${baseUrl}/mods?category_id=1`, "Mods with category_id"],
        [`${baseUrl}/mods/category/1`, "Mods by category path"],

        // Search with empty/broad terms
        [`${baseUrl}/mods/search?q=`, "Search with empty query"],
        [`${baseUrl}/mods/search?query=`, "Search with empty query (alt)"],
        [`${baseUrl}/search?q=`, "Search endpoint"],

        // List/index variations
        [`${baseUrl}/mods/list`, "Mods list endpoint"],
        [`${baseUrl}/mods/index`, "Mods index endpoint"],
        [`${baseUrl}/mods/all`, "All mods endpoint"],

        // Sort/order variations
        [`${baseUrl}/mods?sort=date`, "Mods sorted by date"],
        [`${baseUrl}/mods?order=creation_date`, "Mods ordered by creation"],
        [`${baseUrl}/mods?sort=popular`, "Mods sorted by popularity"],

        // Working endpoints (for comparison)
        [`${baseUrl}/mods/updated.json?period=1m`, "Known working: updated mods"],
        [`${baseUrl}/mods/trending.json`, "Known working: trending mods"],
    ];

    for (const [url, description] of endpointsToTest) {
        await testEndpoint(url, description);
    }

    console.log("\n🏁 Endpoint discovery complete!");
    console.log("\nLook for endpoints that returned arrays with many items or objects with 'total' counts.");
}

// Run the discovery
discoverEndpoints().catch(error => {
    console.error("Discovery failed:", error);
    process.exit(1);
});