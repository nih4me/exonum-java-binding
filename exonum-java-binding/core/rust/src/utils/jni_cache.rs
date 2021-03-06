// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Caching some of the often used methods and classes helps to improve
//! performance. Caching is done immediately after loading of the native
//! library by JVM. To do so, we use JNI_OnLoad method. JNI_OnUnload is not
//! currently used because we don't need to reload native library multiple times
//! during execution.
//!
//! See: https://docs.oracle.com/en/java/javase/12/docs/specs/jni/invocation.html#jni_onload

use std::{os::raw::c_void, panic::catch_unwind};

use jni::{
    objects::{GlobalRef, JMethodID},
    sys::{jint, JNI_VERSION_1_8},
    JNIEnv, JavaVM,
};
use log::debug;
use parking_lot::Once;

/// Invalid JNI version constant, signifying JNI_OnLoad failure.
const INVALID_JNI_VERSION: jint = 0;
const SERVICE_RUNTIME_ADAPTER_CLASS: &str = "com/exonum/binding/core/runtime/ServiceRuntimeAdapter";

static INIT: Once = Once::new();

static mut OBJECT_GET_CLASS: Option<JMethodID> = None;
static mut CLASS_GET_NAME: Option<JMethodID> = None;
static mut THROWABLE_GET_MESSAGE: Option<JMethodID> = None;
static mut THROWABLE_GET_CAUSE: Option<JMethodID> = None;
static mut EXECUTION_EXCEPTION_GET_ERROR_CODE: Option<JMethodID> = None;

static mut RUNTIME_ADAPTER_INITIALIZE: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_DEPLOY_ARTIFACT: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_IS_ARTIFACT_DEPLOYED: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_INITIATE_ADDING_SERVICE: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_INITIATE_RESUMING_SERICE: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_UPDATE_SERVICE_STATUS: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_EXECUTE_TX: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_BEFORE_TRANSACTIONS: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_AFTER_TRANSACTIONS: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_AFTER_COMMIT: Option<JMethodID> = None;
static mut RUNTIME_ADAPTER_SHUTDOWN: Option<JMethodID> = None;

static mut JAVA_LANG_ERROR: Option<GlobalRef> = None;
static mut JAVA_LANG_RUNTIME_EXCEPTION: Option<GlobalRef> = None;
static mut JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION: Option<GlobalRef> = None;
static mut EXECUTION_EXCEPTION: Option<GlobalRef> = None;
static mut UNEXPECTED_EXECUTION_EXCEPTION: Option<GlobalRef> = None;

/// This function is executed on loading native library by JVM.
/// It initializes the cache of method and class references.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint {
    let env = vm.get_env().expect("Cannot get reference to the JNIEnv");

    catch_unwind(|| {
        init_cache(&env);
        JNI_VERSION_1_8
    })
    .unwrap_or(INVALID_JNI_VERSION)
}

/// Initializes JNI cache considering synchronization
pub fn init_cache(env: &JNIEnv) {
    INIT.call_once(|| unsafe { cache_methods(env) });
}

/// Caches all required classes and methods ids.
unsafe fn cache_methods(env: &JNIEnv) {
    OBJECT_GET_CLASS = get_method_id(&env, "java/lang/Object", "getClass", "()Ljava/lang/Class;");
    CLASS_GET_NAME = get_method_id(&env, "java/lang/Class", "getName", "()Ljava/lang/String;");
    THROWABLE_GET_MESSAGE = get_method_id(
        &env,
        "java/lang/Throwable",
        "getMessage",
        "()Ljava/lang/String;",
    );
    THROWABLE_GET_CAUSE = get_method_id(
        &env,
        "java/lang/Throwable",
        "getCause",
        "()Ljava/lang/Throwable;",
    );
    EXECUTION_EXCEPTION_GET_ERROR_CODE = get_method_id(
        &env,
        "com/exonum/binding/core/service/ExecutionException",
        "getErrorCode",
        "()B",
    );
    RUNTIME_ADAPTER_INITIALIZE =
        get_method_id(&env, SERVICE_RUNTIME_ADAPTER_CLASS, "initialize", "(J)V");
    RUNTIME_ADAPTER_DEPLOY_ARTIFACT = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "deployArtifact",
        "([B[B)V",
    );
    RUNTIME_ADAPTER_IS_ARTIFACT_DEPLOYED = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "isArtifactDeployed",
        "([B)Z",
    );
    RUNTIME_ADAPTER_INITIATE_ADDING_SERVICE = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "initiateAddingService",
        "(J[B[B)V",
    );
    RUNTIME_ADAPTER_INITIATE_RESUMING_SERICE = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "initiateResumingService",
        "(J[B[B)V",
    );
    RUNTIME_ADAPTER_UPDATE_SERVICE_STATUS = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "updateServiceStatus",
        "([B[B)V",
    );
    RUNTIME_ADAPTER_EXECUTE_TX = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "executeTransaction",
        "(ILjava/lang/String;I[BJI[B[B)V",
    );
    RUNTIME_ADAPTER_BEFORE_TRANSACTIONS = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "beforeTransactions",
        "(IJ)V",
    );
    RUNTIME_ADAPTER_AFTER_TRANSACTIONS = get_method_id(
        &env,
        SERVICE_RUNTIME_ADAPTER_CLASS,
        "afterTransactions",
        "(IJ)V",
    );
    RUNTIME_ADAPTER_AFTER_COMMIT =
        get_method_id(&env, SERVICE_RUNTIME_ADAPTER_CLASS, "afterCommit", "(JIJ)V");
    RUNTIME_ADAPTER_SHUTDOWN =
        get_method_id(&env, SERVICE_RUNTIME_ADAPTER_CLASS, "shutdown", "()V");

    JAVA_LANG_ERROR = get_class(env, "java/lang/Error");
    JAVA_LANG_RUNTIME_EXCEPTION = get_class(env, "java/lang/RuntimeException");
    JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION = get_class(env, "java/lang/IllegalArgumentException");
    EXECUTION_EXCEPTION = get_class(env, "com/exonum/binding/core/service/ExecutionException");
    UNEXPECTED_EXECUTION_EXCEPTION = get_class(
        env,
        "com/exonum/binding/core/runtime/UnexpectedExecutionException",
    );

    debug!("Done caching references to Java classes and methods.");
}

/// Produces `JMethodID` for a particular method dealing with its lifetime.
///
/// Always returns `Some(method_id)`, panics if method not found.
fn get_method_id(env: &JNIEnv, class: &str, name: &str, sig: &str) -> Option<JMethodID<'static>> {
    let method_id = env
        .get_method_id(class, name, sig)
        // we need this line to erase lifetime in order to save underlying raw pointer in static
        .map(|mid| mid.into_inner().into())
        .unwrap_or_else(|_| {
            panic!(
                "Method {} with signature {} of class {} not found",
                name, sig, class
            )
        });
    Some(method_id)
}

/// Returns cached class reference.
///
/// Always returns Some(class_ref), panics if class not found.
fn get_class(env: &JNIEnv, class: &str) -> Option<GlobalRef> {
    let class = env
        .find_class(class)
        .unwrap_or_else(|_| panic!("Class {} not found", class));
    Some(env.new_global_ref(class).unwrap())
}

fn check_cache_initialized() {
    if !INIT.state().done() {
        panic!("JNI cache is not initialized")
    }
}

/// Refers to the cached methods of the `ServiceRuntimeAdapter` class.
pub mod runtime_adapter {
    use super::*;

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.initialize()`.
    pub fn initialize_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_INITIALIZE.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.deployArtifact()`.
    pub fn deploy_artifact_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_DEPLOY_ARTIFACT.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.isArtifactDeployed()`.
    pub fn is_artifact_deployed_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_IS_ARTIFACT_DEPLOYED.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.initiateAddingService()`.
    pub fn initiate_adding_service_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_INITIATE_ADDING_SERVICE.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.initiateResumingService()`.
    pub fn initiate_resuming_service_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_INITIATE_RESUMING_SERICE.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.updateServiceStatus()`.
    pub fn update_service_status_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_UPDATE_SERVICE_STATUS.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.executeTransaction()`.
    pub fn execute_tx_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_EXECUTE_TX.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.beforeTransactions()`.
    pub fn before_transactions_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_BEFORE_TRANSACTIONS.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.afterTransactions()`.
    pub fn after_transactions_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_AFTER_TRANSACTIONS.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.afterCommit()`.
    pub fn after_commit_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_AFTER_COMMIT.unwrap() }
    }

    /// Returns cached `JMethodID` for `ServiceRuntimeAdapter.shutdown()`.
    pub fn shutdown_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { RUNTIME_ADAPTER_SHUTDOWN.unwrap() }
    }
}

/// Refers to the cached methods of the `java.lang.Object` class.
pub mod object {
    use super::*;

    /// Returns cached `JMethodID` for `java.lang.Object.getClass()`.
    pub fn get_class_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { OBJECT_GET_CLASS.unwrap() }
    }
}

/// Refers to the cached methods of the `java.lang.Class` class.
pub mod class {
    use super::*;

    /// Returns cached `JMethodID` for `java.lang.Class.getName()`.
    pub fn get_name_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { CLASS_GET_NAME.unwrap() }
    }
}

/// Refers to the cached methods of the `java.lang.Throwable` class.
pub mod throwable {
    use super::*;

    /// Returns cached `JMethodID` for `java.lang.Throwable.getMessage()`.
    pub fn get_message_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { THROWABLE_GET_MESSAGE.unwrap() }
    }

    /// Returns cached `JMethodID` for `java.lang.Throwable.getCause()`.
    pub fn get_cause_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { THROWABLE_GET_CAUSE.unwrap() }
    }
}

/// Refers to the cached methods of the `com.exonum.binding.core.transaction.ExecutionException` class.
pub mod execution_exception {
    use super::*;

    /// Returns cached `JMethodID` for `ExecutionException.getErrorCode()`.
    pub fn get_error_code_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { EXECUTION_EXCEPTION_GET_ERROR_CODE.unwrap() }
    }
}

/// Provides access to various cached classes.
pub mod classes_refs {
    use super::*;

    /// Returns cached `JClass` for `java/lang/Error` as a `GlobalRef`.
    pub fn java_lang_error() -> GlobalRef {
        check_cache_initialized();
        unsafe { JAVA_LANG_ERROR.clone().unwrap() }
    }

    /// Returns cached `JClass` for `java/lang/RuntimeException` as a `GlobalRef`.
    pub fn java_lang_runtime_exception() -> GlobalRef {
        check_cache_initialized();
        unsafe { JAVA_LANG_RUNTIME_EXCEPTION.clone().unwrap() }
    }

    /// Returns cached `JClass` for `java/lang/IllegalArgumentException` as a `GlobalRef`.
    pub fn java_lang_illegal_argument_exception() -> GlobalRef {
        check_cache_initialized();
        unsafe { JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION.clone().unwrap() }
    }

    /// Returns cached `JClass` for `ExecutionException` as a `GlobalRef`.
    pub fn execution_exception() -> GlobalRef {
        check_cache_initialized();
        unsafe { EXECUTION_EXCEPTION.clone().unwrap() }
    }

    /// Returns cached `JClass` for `UnexpectedExecutionException` as a `GlobalRef`.
    pub fn unexpected_execution_exception() -> GlobalRef {
        check_cache_initialized();
        unsafe { UNEXPECTED_EXECUTION_EXCEPTION.clone().unwrap() }
    }
}
