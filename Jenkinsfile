// ============================================================================
// grass.ai CI/CD - Jenkins pipeline
// Supports platform=android and platform=alinux builds with tests + examples.
// Compatible with Docker agents; falls back to any agent with Rust installed.
// ============================================================================

pipeline {
    agent any

    options {
        timestamps()
        ansiColor('xterm')
        timeout(time: 30, unit: 'MINUTES')
        disableConcurrentBuilds()
    }

    parameters {
        choice(
            name: 'PLATFORM',
            choices: ['alinux', 'android'],
            description: 'Target build platform'
        )
        booleanParam(
            name: 'SKIP_TESTS',
            defaultValue: false,
            description: 'Skip the test stage (debug only)'
        )
    }

    environment {
        PLATFORM  = "${params.PLATFORM}"
        BUILD_MODE = 'release'
        CARGO_TERM_COLOR = 'always'
        CACHE_PATH = "${WORKSPACE}/.cargo-cache"
        CARGO_HOME = "${WORKSPACE}/.cargo-home"
    }

    stages {
        stage('Prepare') {
            steps {
                echo "grass.ai pipeline: platform=${params.PLATFORM}"

                script {
                    if (env.JENKINS_HOME) {
                        // Inline cache dirs so the container can write
                        sh 'mkdir -p ${CACHE_PATH} ${CARGO_HOME} ${WORKSPACE}/target'
                    }
                }

                // Ensure the top-level Makefile is our single source of truth.
                sh 'make print-platform PLATFORM=${PLATFORM}'

                sh 'rustc --version && cargo --version && rustup show active-toolchain'

                // Android-specific tooling
                script {
                    if (params.PLATFORM == 'android') {
                        sh '''
                            rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
                            if ! command -v cargo-ndk > /dev/null; then
                                cargo install cargo-ndk --locked
                            fi
                        '''
                    }
                }
            }
        }

        stage('Format') {
            steps {
                sh 'make fmt PLATFORM=${PLATFORM}'
            }
        }

        stage('Clippy') {
            steps {
                sh 'make clippy PLATFORM=${PLATFORM}'
            }
        }

        stage('Build') {
            steps {
                sh 'make build PLATFORM=${PLATFORM} BUILD_MODE=${BUILD_MODE}'
            }
        }

        stage('Test') {
            when {
                expression { return !params.SKIP_TESTS }
            }
            steps {
                sh 'make test PLATFORM=${PLATFORM} BUILD_MODE=${BUILD_MODE}'
                // Per-module unit tests + integration tests (with real+mock data)
                sh 'cargo test --test integration_tests -- --nocapture'
                sh 'cargo test --test module_unit_tests'
            }
        }

        stage('Examples') {
            when {
                expression { return params.PLATFORM == 'alinux' }
            }
            steps {
                sh 'make example-examples BUILD_MODE=${BUILD_MODE}'
            }
        }

        stage('Archive Artifacts') {
            steps {
                archiveArtifacts artifacts: 'target/release/**', fingerprint: true, excludes: 'target/release/build/**'
                junit 'target/test-results/**/*.xml' // ignored if no JUnit produced
            }
        }
    }

    post {
        success {
            echo 'grass.ai build completed successfully'
        }
        failure {
            echo 'grass.ai build FAILED - see console output'
        }
        always {
            cleanWs() // keep workspace tidy between runs
        }
    }
}