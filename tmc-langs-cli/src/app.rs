//! Create clap app

use crate::output::UpdatedExercise;
use clap::{App, AppSettings, Arg, SubCommand};
use schemars::JsonSchema;
use std::path::PathBuf;
use tmc_langs::{
    CombinedCourseData, CourseData, CourseDetails, CourseExercise,
    DownloadOrUpdateCourseExercisesResult, ExerciseDesc, ExerciseDetails,
    ExercisePackagingConfiguration, LocalExercise, NewSubmission, Organization, Review, RunResult,
    StyleValidationResult, Submission, SubmissionFeedbackResponse, SubmissionFinished,
    UpdateResult,
};
// use tmc_langs_util::task_executor::RefreshData;

/// Constructs the CLI root.
pub fn create_app() -> App<'static, 'static> {
    // subcommand definitions are alphabetically ordered
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("pretty")
            .help("Pretty-prints all output")
            .long("pretty"))

        .subcommand(SubCommand::with_name("checkstyle")
            .about("Checks the code style for the given exercise")
            .long_about(schema_leaked::<Option<StyleValidationResult>>())
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the project resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("locale")
                .help("Locale as a three letter ISO 639-3 code, e.g. 'eng' or 'fin'.")
                .long("locale")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("If defined, the check results will be written to this path. Overwritten if it already exists.")
                .long("output-path")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("clean")
            .about("Cleans the target exercise using the appropriate language plugin")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("compress-project")
            .about("Compresses the target exercise into a ZIP. Only includes student files using the student file policy of the exercise's plugin")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("Path to the output ZIP archive. Overwritten if it already exists.")
                .long("output-path")
                .required(true)
                .takes_value(true)))

        .subcommand(create_core_app()) // "core"

        /*
        .subcommand(
            SubCommand::with_name("disk-space")
                .about("Returns the amount of free disk space in megabytes left on the partition that contains the given path")
                .arg(Arg::with_name("path")
                    .help("A path in the partition that should be inspected.")
                    .long("path")
                    .required(true)
                    .takes_value(true))
        )
        */

        .subcommand(SubCommand::with_name("extract-project")
            .about("Extracts an exercise from a ZIP archive. If the output-path is a project root, the plugin's student file policy will be used to avoid overwriting student files")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("archive-path")
                .help("Path to the ZIP archive.")
                .long("archive-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("Path to the directory where the archive will be extracted.")
                .long("output-path")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("fast-available-points")
            .about("Parses @Points notations from an exercise's exercise files and returns the point names found")
            .long_about(schema_leaked::<Vec<String>>())
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the projects reside.")
                .long("exercise-path")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("find-exercises")
            .about("Finds all exercise root directories inside the exercise-path")
            .long_about(schema_leaked::<Vec<PathBuf>>())
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the projects reside.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("If given, the search results will be written to this path. Overwritten if it already exists.")
                .long("output-path")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-exercise-packaging-configuration")
            .about("Returns a configuration which separately lists the student files and exercise files inside the given exercise")
            .long_about(schema_leaked::<ExercisePackagingConfiguration>())
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("If given, the configuration will be written to this path. Overwritten if it already exists.")
                .long("output-path")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("list-local-course-exercises")
            .about("Returns a list of local exercises for the given course")
            .long_about(schema_leaked::<Vec<LocalExercise>>())
            .arg(Arg::with_name("client-name")
                .help("The client for which exercises should be listed.")
                .long("client-name")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("course-slug")
                .help("The course slug the local exercises of which should be listed.")
                .long("course-slug")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("prepare-solutions")
            .about("Processes the exercise files in exercise-path, removing all code marked as stubs")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("Path to the directory where the processed files will be written.")
                .long("output-path")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("prepare-stubs")
            .about("Processes the exercise files in exercise-path, removing all code marked as solutions")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("Path to the directory where the processed files will be written.")
                .long("output-path")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("prepare-submission")
            .about("Takes a submission ZIP archive and turns it into an archive with reset test files, and tmc-params, ready for further processing")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("output-format")
                .help("The output format of the submission archive. Defaults to tar.")
                .long("output-format")
                .default_value("tar")
                .possible_values(&["tar", "zip", "zstd"]))
            .arg(Arg::with_name("clone-path")
                .help("Path to exercise's clone path, where the unmodified test files will be copied from.")
                .long("clone-path")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("output-path")
                .help("Path to the resulting archive. Overwritten if it already exists.")
                .long("output-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("stub-zip-path")
                .help("If given, the tests will be copied from this stub ZIP instead, effectively ignoring hidden tests.")
                .long("stub-zip-path")
                .takes_value(true))
            .arg(Arg::with_name("submission-path")
                .help("Path to the submission ZIP archive.")
                .long("submission-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("tmc-param")
                .help("A key-value pair in the form key=value to be written into .tmcparams. If multiple pairs with the same key are given, the values are collected into an array.")
                .long("tmc-param")
                .takes_value(true)
                .multiple(true))
            .arg(Arg::with_name("top-level-dir-name")
                .help("If given, the contents in the resulting archive will be nested inside a directory with this name.")
                .long("top-level-dir-name")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("refresh-course")
            .about("Refresh the given course")
            // .long_about(schema_leaked::<RefreshData>()) // can't format YAML mapping
            .arg(Arg::with_name("cache-path")
                .help("Path to the cached course.")
                .long("cache-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("cache-root")
                .help("The cache root.")
                .long("cache-root")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("course-name")
                .help("The name of the course.")
                .long("course-name")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("git-branch")
                .help("Version control branch.")
                .long("git-branch")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("source-url")
                .help("Version control URL.")
                .long("source-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("run-tests")
            .about("Run the tests for the exercise using the appropriate language plugin")
            .long_about(schema_leaked::<RunResult>())
            .arg(Arg::with_name("checkstyle-output-path")
                .help("Runs checkstyle if given. Path to the file where the style results will be written.")
                .long("checkstyle-output-path")
                .takes_value(true)
                .requires("locale"))
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the exercise resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("locale")
                .help("Language as a three letter ISO 639-3 code, e.g. 'eng' or 'fin'. Required if checkstyle-output-path is given.")
                .long("locale")
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("If defined, the test results will be written to this path. Overwritten if it already exists.")
                .long("output-path")
                .takes_value(true))
            .arg(Arg::with_name("wait-for-secret")
                .help("If defined, the command will wait for a string to be written to stdin, used for signing the output file with jwt.")
                .long("wait-for-secret")))

        .subcommand(create_settings_app()) // "settings"

        .subcommand(SubCommand::with_name("scan-exercise")
            .about("Produces a description of an exercise using the appropriate language plugin")
            .long_about(schema_leaked::<ExerciseDesc>())
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the project resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("If given, the scan results will be written to this path. Overwritten if it already exists.")
                .long("output-path")
                .takes_value(true)))
}

/// Constructs the core sub-command.
fn create_core_app() -> App<'static, 'static> {
    App::new("core")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Various commands that communicate with the TMC server")
        .arg(Arg::with_name("client-name")
            .help("Name used to differentiate between different TMC clients.")
            .long("client-name")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("client-version")
            .help("Client version.")
            .long("client-version")
            .required(true)
            .takes_value(true))

        .subcommand(SubCommand::with_name("check-exercise-updates")
            .about("Checks for updates to any exercises that exist locally.")
            .long_about(schema_leaked::<Vec<UpdatedExercise>>()))

        .subcommand(SubCommand::with_name("download-model-solution")
            .about("Downloads an exercise's model solution")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("solution-download-url")
                .help("URL to the solution download.")
                .long("solution-download-url")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("target")
                .help("Path to where the model solution will be downloaded.")
                .long("target")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("download-old-submission")
            .about("Downloads an old submission. Resets the exercise at output-path if any, downloading the exercise base from the server. The old submission is then downloaded and extracted on top of the base, using the student file policy to retain student files")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("save-old-state") // TODO: unnecessary, remove (submission-url is enough, but probaly needs a rename if the flag is removed)
                .help("If set, the exercise is submitted to the server before resetting it.")
                .long("save-old-state")
                .requires("submission-url"))
            .arg(Arg::with_name("exercise-id")
                .help("The ID of the exercise.")
                .long("exercise-id")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("output-path")
                .help("Path to where the submission should be downloaded.")
                .long("output-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-id")
                .help("The ID of the submission.")
                .long("submission-id")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-url")
                .help("Required if save-old-state is set. The URL where the submission should be posted.")
                .long("submission-url")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("download-or-update-course-exercises")
            .about("Downloads exercises. If downloading an exercise that has been downloaded before, the student file policy will be used to avoid overwriting student files, effectively just updating the exercise files")
            .long_about(schema_leaked::<DownloadOrUpdateCourseExercisesResult>())
            .arg(Arg::with_name("download-template")
                .help("If set, will always download the course template instead of the latest submission, even if one exists.")
                .long("download-template"))
            .arg(Arg::with_name("exercise-id")
                .help("Exercise id of an exercise that should be downloaded. Multiple ids can be given.")
                .long("exercise-id")
                .required(true)
                .takes_value(true)
                .multiple(true)))

        .subcommand(SubCommand::with_name("get-course-data")
            .about("Fetches course data. Combines course details, course exercises and course settings")
            .long_about(schema_leaked::<CombinedCourseData>())
            .arg(Arg::with_name("course-id")
                .help("The ID of the course.")
                .long("course-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-course-details")
            .about("Fetches course details")
            .long_about(schema_leaked::<CourseDetails>())
            .arg(Arg::with_name("course-id")
                .help("The ID of the course.")
                .long("course-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-course-exercises")
            .about("Lists a course's exercises")
            .long_about(schema_leaked::<Vec<CourseExercise>>())
            .arg(Arg::with_name("course-id")
                .help("The ID of the course.")
                .long("course-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-course-settings")
            .about("Fetches course settings")
            .long_about(schema_leaked::<CourseData>())
            .arg(Arg::with_name("course-id")
                .help("The ID of the course.")
                .long("course-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-courses")
            .about("Lists courses")
            .long_about(schema_leaked::<Vec<CourseData>>())
            .arg(Arg::with_name("organization")
                .help("Organization slug (e.g. mooc, hy).")
                .long("organization")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-exercise-details")
            .about("Fetches exercise details")
            .long_about(schema_leaked::<ExerciseDetails>())
            .arg(Arg::with_name("exercise-id")
                .help("The ID of the exercise.")
                .long("exercise-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-exercise-submissions")
            .about("Fetches the current user's old submissions for an exercise")
            .long_about(schema_leaked::<Vec<Submission>>())
            .arg(Arg::with_name("exercise-id")
                .help("The ID of the exercise.")
                .long("exercise-id")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-exercise-updates")
            .about("Checks for updates to exercises")
            .long_about(schema_leaked::<UpdateResult>())
            .arg(Arg::with_name("course-id")
                .help("The ID of the course")
                .long("course-id")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("exercise")
                .help("An exercise. Takes two values, an exercise id and a checksum. Multiple exercises can be given.")
                .long("exercise")
                .required(true)
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["exercise-id", "checksum"])
                .multiple(true)))

        .subcommand(SubCommand::with_name("get-organization")
            .about("Fetches an organization")
            .long_about(schema_leaked::<Organization>())
            .arg(Arg::with_name("organization")
                .help("Organization slug (e.g. mooc, hy).")
                .long("organization")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("get-organizations")
            .about("Fetches a list of all organizations from the TMC server")
            .long_about(schema_leaked::<Vec<Organization>>()))

        .subcommand(SubCommand::with_name("get-unread-reviews")
            .about("Fetches unread reviews")
            .long_about(schema_leaked::<Vec<Review>>())
            .arg(Arg::with_name("reviews-url")
                .help("URL to the reviews API.")
                .long("reviews-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("logged-in")
            .about("Checks if the CLI is authenticated. Prints the access token if so")
            .long_about(SCHEMA_TOKEN))

        .subcommand(SubCommand::with_name("login")
            .about("Authenticates with the TMC server and stores the OAuth2 token in config. You can log in either by email and password or an access token")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("base64")
                .help("If set, the password is expected to be a base64 encoded string. This can be useful if the password contains special characters.")
                .long("base64"))
            .arg(Arg::with_name("email")
                .help("The email address of your TMC account. The password will be read through stdin.")
                .long("email")
                .takes_value(true)
                .required_unless("set-access-token"))
            .arg(Arg::with_name("set-access-token")
                .help("The OAUTH2 access token that should be used for authentication.")
                .long("set-access-token")
                .takes_value(true)
                .required_unless("email")))

        .subcommand(SubCommand::with_name("logout")
            .about("Logs out and removes the OAuth2 token from config")
            .long_about(SCHEMA_NULL))

        .subcommand(SubCommand::with_name("mark-review-as-read")
            .about("Marks a review as read")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("review-update-url")
                .help("URL to the review update API.")
                .long("review-update-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("paste")
            .about("Sends an exercise to the TMC pastebin")
            .long_about(schema_leaked::<NewSubmission>())
            .arg(Arg::with_name("locale")
                .help("Language as a three letter ISO 639-3 code, e.g. 'eng' or 'fin'.")
                .long("locale")
                .takes_value(true))
            .arg(Arg::with_name("paste-message")
                .help("Optional message to attach to the paste.")
                .long("paste-message")
                .takes_value(true))
            .arg(Arg::with_name("submission-path")
                .help("Path to the exercise to be submitted.")
                .long("submission-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-url")
                .help("The URL where the submission should be posted.")
                .long("submission-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("request-code-review")
            .about("Requests code review")
            .long_about(schema_leaked::<NewSubmission>())
            .arg(Arg::with_name("locale")
                .help("Language as a three letter ISO 639-3 code, e.g. 'eng' or 'fin'.")
                .long("locale")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("message-for-reviewer")
                .help("Message for the review.")
                .long("message-for-reviewer")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-path")
                .help("Path to the directory where the submission resides.")
                .long("submission-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-url")
                .help("URL where the submission should be posted.")
                .long("submission-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("reset-exercise")
            .about("Resets an exercise. Removes the contents of the exercise directory and redownloads it from the server")
            .long_about(SCHEMA_NULL)
            .arg(Arg::with_name("save-old-state")
                .help("If set, the exercise is submitted to the server before resetting it.")
                .long("save-old-state")
                .requires("submission-url"))
            .arg(Arg::with_name("exercise-id")
                .help("The ID of the exercise.")
                .long("exercise-id")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("exercise-path")
                .help("Path to the directory where the project resides.")
                .long("exercise-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-url")
                .help("Required if save-old-state is set. The URL where the submission should be posted.")
                .long("submission-url")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("send-feedback")
            .about("Sends feedback for an exercise")
            .long_about(schema_leaked::<SubmissionFeedbackResponse>())
            .arg(Arg::with_name("feedback")
                .help("A feedback answer. Takes two values, a feedback answer id and the answer. Multiple feedback arguments can be given.")
                .long("feedback")
                .required(true)
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["feedback-answer-id", "answer"])
                .multiple(true))
            .arg(Arg::with_name("feedback-url")
                .help("URL where the feedback should be posted.")
                .long("feedback-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("submit")
            .about("Submits an exercise. By default blocks until the submission results are returned")
            .long_about(schema_leaked::<SubmissionFinished>())
            .arg(Arg::with_name("dont-block")
                .help("Set to avoid blocking.")
                .long("dont-block"))
            .arg(Arg::with_name("locale")
                .help("Language as a three letter ISO 639-3 code, e.g. 'eng' or 'fin'.")
                .long("locale")
                .takes_value(true))
            .arg(Arg::with_name("submission-path")
                .help("Path to the directory where the exercise resides.")
                .long("submission-path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("submission-url")
                .help("URL where the submission should be posted.")
                .long("submission-url")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("update-exercises")
            .about("Updates all local exercises that have been updated on the server")
            .long_about(SCHEMA_NULL))

        .subcommand(SubCommand::with_name("wait-for-submission")
            .about("Waits for a submission to finish")
            .long_about(schema_leaked::<SubmissionFinished>())
            .arg(Arg::with_name("submission-url")
                .help("URL to the submission's status.")
                .long("submission-url")
                .required(true)
                .takes_value(true)))
}

fn create_settings_app() -> App<'static, 'static> {
    App::new("settings")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Configure the CLI")
        .arg(
            Arg::with_name("client-name")
                .help("The name of the client.")
                .long("client-name")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Retrieves a value from the settings")
                .arg(
                    Arg::with_name("setting")
                        .help("The name of the setting.")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list").about("Prints every key=value pair in the settings file"),
        )
        .subcommand(
            SubCommand::with_name("migrate")
                .about("Migrates an exercise on disk into the langs project directory")
                .arg(
                    Arg::with_name("exercise-path")
                        .help("Path to the directory where the project resides.")
                        .long("exercise-path")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("course-slug")
                        .help("The course slug, e.g. mooc-java-programming-i.")
                        .long("course-slug")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("exercise-id")
                        .help("The exercise id, e.g. 1234.")
                        .long("exercise-id")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("exercise-slug")
                        .help("The exercise slug, e.g. part01-Part01_01.Sandbox.")
                        .long("exercise-slug")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("exercise-checksum")
                        .help("The checksum of the exercise from the TMC server.")
                        .long("exercise-checksum")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("move-projects-dir")
                .about(
                    "Change the projects-dir setting, moving the contents into the new directory",
                )
                .arg(
                    Arg::with_name("dir")
                        .help("The directory where the projects should be moved.")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("reset").about("Resets the settings file to the defaults"),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Saves a value in the settings")
                .arg(
                    Arg::with_name("key")
                        .help("The key. Parsed as JSON, assumed to be a string if parsing fails.")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("json")
                        .help("The value in JSON.")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("unset")
                .about("Unsets a value from the settings")
                .arg(
                    Arg::with_name("setting")
                        .help("The name of the setting.")
                        .required(true)
                        .takes_value(true),
                ),
        )
}

// == utilities for printing the JSON schema of the objects printed to stdout by the CLI ==
const SCHEMA_NULL: &str = "Result data JSON format: null";
const SCHEMA_TOKEN: &str = r#"Result data JSON format:
{
    "access_token": String,
    "token_type": String,
    "scope": String,
}"#;

// clap's long_about only accepts string slices, so
// this function is used to leak a constant amount of
// memory to dynamically create static slices
// todo: stop leaking memory
fn schema_leaked<T: JsonSchema>() -> &'static str {
    let schema = schemars::schema_for!(T);
    let json = format!(
        "Result data JSON format:\n{}",
        serde_json::to_string_pretty(&schema).unwrap()
    );
    Box::leak(Box::new(json))
}

#[cfg(test)]
mod base_test {
    use super::*;

    fn get_matches(args: &[&str]) {
        create_app().get_matches_from(&["tmc-langs-cli"].iter().chain(args).collect::<Vec<_>>());
    }

    #[test]
    fn sanity() {
        assert!(create_app()
            .get_matches_from_safe(&["tmc-langs-cli", "checkstyle", "--non-existent-arg"])
            .is_err());
    }

    #[test]
    fn checkstyle() {
        get_matches(&[
            "checkstyle",
            "--exercise-path",
            "path",
            "--locale",
            "fi",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn clean() {
        get_matches(&["clean", "--exercise-path", "path"]);
    }

    #[test]
    fn compress_project() {
        get_matches(&[
            "compress-project",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    // #[test]
    fn disk_space() {
        get_matches(&["disk-space", "--path", "path"]);
    }

    #[test]
    fn extract_project() {
        get_matches(&[
            "extract-project",
            "--archive-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn fast_available_points() {
        get_matches(&["fast-available-points", "--exercise-path", "path"]);
    }

    #[test]
    fn find_exercises() {
        get_matches(&[
            "find-exercises",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn get_exercise_packaging_configuration() {
        get_matches(&[
            "get-exercise-packaging-configuration",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn list_local_course_exercises() {
        get_matches(&[
            "list-local-course-exercises",
            "--client-name",
            "client",
            "--course-slug",
            "slug",
        ]);
    }

    #[test]
    fn prepare_solutions() {
        get_matches(&[
            "prepare-solutions",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn prepare_stubs() {
        get_matches(&[
            "prepare-stubs",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn prepare_submission() {
        get_matches(&[
            "prepare-submission",
            "--clone-path",
            "path",
            "--output-format",
            "tar",
            "--output-path",
            "path",
            "--stub-zip-path",
            "path",
            "--submission-path",
            "path",
            "--tmc-param",
            "a=b",
            "--tmc-param",
            "c=d",
        ]);
    }

    #[test]
    fn refresh_course() {
        get_matches(&[
            "refresh-course",
            "--cache-path",
            "path",
            "--cache-root",
            "path",
            "--course-name",
            "name",
            "--git-branch",
            "main",
            "--source-url",
            "example.com",
        ]);
    }

    #[test]
    fn run_tests() {
        get_matches(&[
            "run-tests",
            "--checkstyle-output-path",
            "path",
            "--exercise-path",
            "path",
            "--locale",
            "fi",
            "--output-path",
            "path",
        ]);
    }

    #[test]
    fn scan_exercise() {
        get_matches(&[
            "scan-exercise",
            "--exercise-path",
            "path",
            "--output-path",
            "path",
        ]);
    }
}

#[cfg(test)]
mod core_test {
    use super::*;

    fn get_matches_core(args: &[&str]) {
        create_app().get_matches_from(
            &[
                "tmc-langs-cli",
                "core",
                "--client-name",
                "client",
                "--client-version",
                "version",
            ]
            .iter()
            .chain(args)
            .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn check_exercise_updates() {
        get_matches_core(&["check-exercise-updates"]);
    }

    #[test]
    fn download_model_solution() {
        get_matches_core(&[
            "download-model-solution",
            "--solution-download-url",
            "localhost",
            "--target",
            "path",
        ]);
    }

    #[test]
    fn download_old_submission() {
        get_matches_core(&[
            "download-old-submission",
            "--save-old-state",
            "--exercise-id",
            "1234",
            "--output-path",
            "path",
            "--submission-id",
            "2345",
            "--submission-url",
            "localhost",
        ]);
    }

    #[test]
    fn download_or_update_course_exercises() {
        get_matches_core(&[
            "download-or-update-course-exercises",
            "--exercise-id",
            "1234",
            "--exercise-id",
            "2345",
        ]);
    }

    #[test]
    fn get_course_data() {
        get_matches_core(&["get-course-data", "--course-id", "1234"]);
    }

    #[test]
    fn get_course_details() {
        get_matches_core(&["get-course-details", "--course-id", "1234"]);
    }

    #[test]
    fn get_course_exercises() {
        get_matches_core(&["get-course-exercises", "--course-id", "1234"]);
    }

    #[test]
    fn get_course_settings() {
        get_matches_core(&["get-course-settings", "--course-id", "1234"]);
    }

    #[test]
    fn get_courses() {
        get_matches_core(&["get-courses", "--organization", "org"]);
    }

    #[test]
    fn get_exercise_details() {
        get_matches_core(&["get-exercise-details", "--exercise-id", "1234"]);
    }

    #[test]
    fn get_exercise_submissions() {
        get_matches_core(&["get-exercise-submissions", "--exercise-id", "1234"]);
    }

    #[test]
    fn get_exercise_updates() {
        get_matches_core(&[
            "get-exercise-updates",
            "--course-id",
            "1234",
            "--exercise",
            "1234",
            "abcd",
            "--exercise",
            "2345",
            "bcde",
        ]);
    }

    #[test]
    fn get_organization() {
        get_matches_core(&["get-organization", "--organization", "org"]);
    }

    #[test]
    fn get_organizations() {
        get_matches_core(&["get-organizations"]);
    }

    #[test]
    fn get_unread_reviews() {
        get_matches_core(&["get-unread-reviews", "--reviews-url", "localhost"]);
    }

    #[test]
    fn logged_in() {
        get_matches_core(&["logged-in"]);
    }

    #[test]
    fn login() {
        get_matches_core(&[
            "login",
            "--base64",
            "--email",
            "email",
            "--set-access-token",
            "access token",
        ]);
    }

    #[test]
    fn logout() {
        get_matches_core(&["logout"]);
    }

    #[test]
    fn mark_review_as_read() {
        get_matches_core(&["mark-review-as-read", "--review-update-url", "localhost"]);
    }

    #[test]
    fn paste() {
        get_matches_core(&[
            "paste",
            "--locale",
            "fi",
            "--paste-message",
            "msg",
            "--submission-path",
            "path",
            "--submission-url",
            "localhost",
        ]);
    }

    #[test]
    fn request_code_review() {
        get_matches_core(&[
            "request-code-review",
            "--locale",
            "fi",
            "--message-for-reviewer",
            "msg",
            "--submission-path",
            "path",
            "--submission-url",
            "localhost",
        ]);
    }

    #[test]
    fn reset_exercise() {
        get_matches_core(&[
            "reset-exercise",
            "--save-old-state",
            "--exercise-id",
            "1234",
            "--exercise-path",
            "path",
            "--submission-url",
            "localhost",
        ]);
    }

    #[test]
    fn send_feedback() {
        get_matches_core(&[
            "send-feedback",
            "--feedback",
            "1234",
            "answer",
            "--feedback-url",
            "localhost",
        ]);
    }

    #[test]
    fn submit() {
        get_matches_core(&[
            "submit",
            "--dont-block",
            "--locale",
            "fi",
            "--submission-path",
            "path",
            "--submission-url",
            "localhost",
        ]);
    }

    #[test]
    fn update_exercises() {
        get_matches_core(&["update-exercises"]);
    }

    #[test]
    fn wait_for_submission() {
        get_matches_core(&["wait-for-submission", "--submission-url", "localhost"]);
    }
}

#[cfg(test)]
mod settings_test {
    use super::*;

    fn get_matches_settings(args: &[&str]) {
        create_app().get_matches_from(
            &["tmc-langs-cli", "settings", "--client-name", "client"]
                .iter()
                .chain(args)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn get() {
        get_matches_settings(&["get", "key"]);
    }

    #[test]
    fn list() {
        get_matches_settings(&["list"]);
    }

    #[test]
    fn migrate() {
        get_matches_settings(&[
            "migrate",
            "--exercise-path",
            "path",
            "--course-slug",
            "slug",
            "--exercise-id",
            "1234",
            "--exercise-slug",
            "slug",
            "--exercise-checksum",
            "abcd",
        ]);
    }

    #[test]
    fn move_projects_dir() {
        get_matches_settings(&["move-projects-dir", "path"]);
    }

    #[test]
    fn reset() {
        get_matches_settings(&["reset"]);
    }

    #[test]
    fn set() {
        get_matches_settings(&["set", "key", "json"]);
    }

    #[test]
    fn unset() {
        get_matches_settings(&["unset", "key"]);
    }
}
