const semver = require('semver')

if (process.argv.length !== 4) {
    console.error('Two arguments requred');
    process.exit(1);
}

const old_version = process.argv[2];
const new_version = process.argv[3];

// if (!semver.gt(new_version, old_version)) {
//     console.error('Version is not greater than previous release');
//     process.exit(2);
// }
console.log((semver.gt(new_version, old_version)).toString())
