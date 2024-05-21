// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#include "utils/Mutex.h"

#include "InterCOM/PlatformConfig.h"
#include "utils/BoundedString.h"
#include "utils/Exception.h"
#include "utils/StringFormatter.h"
#include "utils/Thread.h"
#include "utils/Time.h"
#include "utils/Util.h"

#if defined(INTERCOM_POSIX_COMPLIANT)
#  include <fcntl.h>
#  include <pthread.h>
#  include <semaphore.h>

#  include <cerrno>
#endif

#if defined(INTERCOM_PLATFORM_LINUX)
#  include <sys/time.h>
#elif defined(INTERCOM_PLATFORM_WINDOWS)
#  include <windows.h>
#elif defined(INTERCOM_PLATFORM_VXWORKS)
// #  include <bits/pthreadtypes.h>
#  include <semLib.h>
#  include <sysLib.h>
#else
#  error "Platform not supported"
#endif

#include <string>

namespace {

#if defined(INTERCOM_POSIX_COMPLIANT)
#elif defined(INTERCOM_PLATFORM_WINDOWS)
#elif defined(INTERCOM_PLATFORM_VXWORKS)
#else
#  error "Platform not supported"
#endif
}  // namespace

#ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
intercom::dcps::Time_t enter_mutex;
intercom::dcps::Time_t retry_mutex;
intercom::dcps::Time_t throw_mutex;
#endif

namespace intercom {
Mutex::ScopeLock::ScopeLock() : m_mtx(nullptr){};

Mutex::ScopeLock::ScopeLock(Mutex& a_mtx) : m_mtx(nullptr) {
    m_mtx = &a_mtx;
    a_mtx.lock();
};

Mutex::ScopeLock::ScopeLock(Mutex* a_mtx) : m_mtx(nullptr) {
    m_mtx = a_mtx;
    a_mtx->lock();
};

Mutex::ScopeLock::~ScopeLock() {
    if (m_mtx) {
        m_mtx->release();
        m_mtx = nullptr;
    }
}

void Mutex::ScopeLock::lock(Mutex& a_mtx) {
    allthreads_assert(m_mtx == nullptr);
    m_mtx = &a_mtx;
    a_mtx.lock();
}

void Mutex::ScopeLock::lock(Mutex* a_mtx) {
    lock(*a_mtx);
}

void Mutex::ScopeLock::release() {
    allthreads_assert(m_mtx != nullptr);
    if (m_mtx) {
        m_mtx->release();
        m_mtx = nullptr;
    }
};

Mutex::~Mutex() {
#ifdef INTERCOM_MUTEX_DEBUG
    allthreads_assert(m_thread_handle == 0);
#endif
}

#ifdef INTERCOM_MUTEX_DEBUG
// These values only apply to INTERCOM_MUTEX_DEBUG and INTERCOM_ENABLE_SEMAPHORE_MONITORING
// Values should be adjusted according to what is needed for the current debug session.
// Note: if the durations are exceeded the application will abort (in debug mode).
// Current values are for lock-time monitoring.
const dcps::Duration_t WARN_AT_LOCKTIME = {0, 10 * 1000 * 1000};
const dcps::Duration_t WARN_ON_HELDTIME = {0, 100 * 1000 * 1000};
const dcps::Duration_t DEADLOCK_DETECTION_DELAY = {300, 0};
#endif

void Mutex::lock() {
#if defined(INTERCOM_MUTEX_DEBUG)
#  ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
    dcps::Time_t startTime = getSystemTime();
#  endif
    if (trylock(DEADLOCK_DETECTION_DELAY)) {
#  ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
        dcps::Duration_t lockTime = getSystemTime() - startTime;
        if (lockTime > WARN_AT_LOCKTIME) {
            allthreads_assert(false);
        }
#  endif
        return;
    }
    allthreads_assert(false);
    abort();
#else
#  ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
    dcps::Time_t startTime = getSystemTime();
#  endif
    if (!platformLockMutex(m_handle)) {
        intercom::StringFormatter<100> msg;
        msg << "Lock for mutex " << m_name << " could not be acquired - possible deadlock";
        throw MutexDeadlockException(msg.c_str(), SOURCE_LOCATION);
    }
#  ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
    dcps::Duration_t lockTime = getSystemTime() - startTime;
    if (lockTime > WARN_AT_LOCKTIME) {
        allthreads_assert(false);
    }
    m_lockTime = getSystemTime();
#  endif
#endif
#if defined(INTERCOM_MUTEX_DEBUG)
    if (m_thread_handle == 0) {
        m_thread_handle = Thread::current();
    }
    allthreads_assert(m_thread_handle == Thread::current());
    m_count++;
#endif
}

void Mutex::release() {
#ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
    dcps::Time_t unlockTime = getSystemTime();
    dcps::Duration_t deltaT = unlockTime - m_lockTime;
    if (deltaT > WARN_ON_HELDTIME) {
        allthreads_assert(false);
    }
#endif
#if defined(INTERCOM_MUTEX_DEBUG)
    if (--m_count == 0) {
        m_thread_handle = 0;
    }
#endif
    if (!platformUnlockMutex(m_handle)) {
        throw FunctionCallFailedException("Mutex could not be unlocked error", SOURCE_LOCATION);
    }
}

bool Mutex::release_nothrow() {
#ifdef INTERCOM_ENABLE_SEMAPHORE_MONITORING
    dcps::Time_t unlockTime = getSystemTime();
    dcps::Duration_t deltaT = unlockTime - m_lockTime;
    if (deltaT > WARN_ON_HELDTIME) {
        allthreads_assert(false);
    }
#endif
#if defined(INTERCOM_MUTEX_DEBUG)
    if (--m_count == 0) {
        m_thread_handle = 0;
    }
#endif
    return platformUnlockMutex(m_handle);
}

#ifdef _WIN32
#  pragma warning(push)
#  pragma warning(disable : 4355)  // 'this' used in base initializer list
#endif
Mutex::Mutex(const char* a_name, PlatformMutexHandle* a_p_handle)
        : m_handle(a_p_handle),
#ifdef NDEBUG
          m_name(a_name)
#else
          m_strName(intercom::StringFormatter<40>() << a_name << ":" << this),
          m_name(m_strName.c_str())
#endif
#ifdef INTERCOM_MUTEX_DEBUG
          ,
          m_thread_handle(0),
          m_count(0)
#endif
{
}
#ifdef _WIN32
#  pragma warning(pop)
#endif

#if defined(INTERCOM_MUTEX_DEBUG)
void Mutex::ASSERT(bool sameThread) {
    allthreads_assert(sameThread == (m_thread_handle == Thread::current()));
}
#endif

bool Mutex::platformLockMutex(PlatformMutexHandle* a_handle) {
#if defined(INTERCOM_PLATFORM_WINDOWS)
    return (WaitForSingleObject(*a_handle, INFINITE) == WAIT_OBJECT_0);
#elif defined(INTERCOM_PLATFORM_VXWORKS_kernel)
    return (semTake(*a_handle, WAIT_FOREVER) == OK);
#elif defined(INTERCOM_POSIX_COMPLIANT)
    return (pthread_mutex_lock(a_handle) == 0);
#else
#  error "Platform not supported"
#endif
}

bool Mutex::platformTryLockMutex(PlatformMutexHandle* a_handle, const dcps::Duration_t& a_timeout) {
#if defined(INTERCOM_PLATFORM_WINDOWS)
    DWORD millisec = DWORD(a_timeout.sec * 1000) + DWORD(a_timeout.nanosec / 1e6);
    DWORD status = WaitForSingleObject(*a_handle, millisec);
    if (status != WAIT_OBJECT_0) {
        return (status == WAIT_ABANDONED);
    }
    return true;
#elif defined(INTERCOM_PLATFORM_VXWORKS_kernel)
    int ticks = int(a_timeout.sec * sysClkRateGet()) + int(a_timeout.nanosec * sysClkRateGet() / 1e9);
    return (semTake(*a_handle, ticks) == OK);
#elif defined(INTERCOM_POSIX_COMPLIANT)
    dcps::Time_t tryUntil = getSystemTime() + a_timeout;
    timespec ts;
    ts.tv_sec = time_t(tryUntil.sec);
    ts.tv_nsec = tryUntil.nanosec;
    bool rval = (pthread_mutex_timedlock(a_handle, &ts) == 0);
    return rval;
#else
#  error "Platform not supported"
#endif
}

bool Mutex::platformUnlockMutex(PlatformMutexHandle* a_handle) {
#if defined(INTERCOM_PLATFORM_WINDOWS)
    bool rval = (ReleaseMutex(*a_handle) != 0);
#elif defined(INTERCOM_PLATFORM_VXWORKS_kernel)
    bool rval = (semGive(*a_handle) == OK);
#elif defined(INTERCOM_POSIX_COMPLIANT)
    bool rval = (pthread_mutex_unlock(a_handle) == 0);
#else
#  error "Platform not supported"
#endif
    return rval;
}

void Mutex::platformDeleteMutex(PlatformMutexHandle* a_handle) {
#if defined(INTERCOM_PLATFORM_WINDOWS)
    CloseHandle(*a_handle);
#elif defined(INTERCOM_PLATFORM_VXWORKS_kernel)
    semDelete(*a_handle);
#elif defined(INTERCOM_POSIX_COMPLIANT)
    pthread_mutex_destroy(a_handle);
#else
#  error "Platform not supported"
#endif
}

RecursiveMutex::RecursiveMutex(const char* a_name) : Mutex(a_name, &m_mutex) {
    if (!platformCreateMutex(&m_mutex)) {
        throw FunctionCallFailedException("CreateMutex(...) failed", SOURCE_LOCATION);
    }
}

RecursiveMutex::~RecursiveMutex() {
    platformDeleteMutex(&m_mutex);
}

bool RecursiveMutex::platformCreateMutex(PlatformMutexHandle* a_handle) {
#if defined(INTERCOM_PLATFORM_WINDOWS)
    *a_handle = CreateMutex(NULL, FALSE, NULL);
    return *a_handle != NULL;
    /*
#elif defined ( INTERCOM_PLATFORM_VXWORKS_kernel )
    *a_handle = semMCreate( SEM_Q_PRIORITY );
    return *a_handle != NULL;
    */
#elif defined(INTERCOM_POSIX_COMPLIANT)
    pthread_mutexattr_t attr;
    if (pthread_mutexattr_init(&attr) != 0) {
        return false;
    }
    if (pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE) != 0) {
        pthread_mutexattr_destroy(&attr);
        return false;
    }
    if (pthread_mutex_init(a_handle, &attr) != 0) {
        pthread_mutexattr_destroy(&attr);
        return false;
    }
    pthread_mutexattr_destroy(&attr);
    return true;
#else
#  error "Platform not supported"
#endif
}

SharedMutex::SharedMutex(const char* name) : Mutex(name, nullptr), m_locked(false), m_readers(0) {}

void SharedMutex::lock() {
    std::unique_lock<std::mutex> lock(m_mtx);
    while (m_locked) {
        m_shared.wait(lock);
    }
    m_locked = true;
    if (m_readers != 0) {
        m_exclusive.wait(lock);
    }
}

void SharedMutex::release() {
    std::unique_lock<std::mutex> lock(m_mtx);
    m_locked = false;
    m_shared.notify_all();
}

bool SharedMutex::release_nothrow() {
    release();
    return true;
}

bool SharedMutex::try_lock() {
    std::unique_lock<std::mutex> lock(m_mtx);
    if (!m_locked) {
        m_locked = true;
        return true;
    }
    return false;
}

void SharedMutex::lock_shared() {
    std::unique_lock<std::mutex> lock(m_mtx);
    while (m_locked) {
        m_shared.wait(lock);
    }
    m_readers++;
}

bool SharedMutex::try_lock_shared() {
    std::unique_lock<std::mutex> lock(m_mtx);
    if (!m_locked) {
        m_readers++;
        return true;
    }
    return false;
}

void SharedMutex::release_shared() {
    std::unique_lock<std::mutex> lock(m_mtx);
    m_readers--;

    if (m_locked) {
        if (m_readers == 0) {
            m_exclusive.notify_one();
        }
    } else {
        m_shared.notify_one();
    }
}

SharedMutex::SharedLock::SharedLock(SharedMutex& mutex) : m_mtx(mutex) {
    mutex.lock_shared();
}

SharedMutex::SharedLock::~SharedLock() {
    m_mtx.release_shared();
}

}  // namespace intercom
