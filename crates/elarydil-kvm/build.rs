fn main() {
    // Keep the Windows OS UCRT while removing the separate VC redist requirement.
    // This build-only helper does nothing for non-MSVC targets.
    static_vcruntime::metabuild();
}
