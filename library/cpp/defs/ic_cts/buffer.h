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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#pragma once

#include "ic_cts/span.h"

namespace ic_cts {

class Buffer {
  public:
    using const_iterator = const uint8_t*;

    ///
    Buffer();

    ///
    explicit Buffer(size_t length);

    ///
    Buffer(uint8_t* pointer, size_t length);

    ///
    Buffer(const uint8_t* pointer, size_t length);

    ///
    Buffer(Buffer&&) noexcept;

    ///
    Buffer& operator=(const Buffer&) = delete;

    ///
    Buffer(Buffer&) = delete;

    ///
    ~Buffer();

    bool operator==(const Buffer& a_other) const;

    bool operator!=(const Buffer& a_other) const;

    /// Set the buffer to point to an externally contained block of memory.
    /// The buffer instance will NOT deallocate this buffer, and it is the responsibility of the
    /// client code to ensure that this data is valid and avaliable whenever used.
    void set_buffer(uint8_t* pointer, size_t a_length);

    /// Set the buffer from an external source. No size alteration is legal for unowned buffers.
    /// \param pointer new pointer to use as start of buffer
    /// \param a_length number of bytes in the buffer
    void set_buffer(const uint8_t* pointer, size_t a_length);

    /// Allocate memory for the buffer.
    /// \param a_new_length the number of bytes the new buffer should at least contain.
    /// If the buffer already contains that many bytes, the buffer is not reallocated.
    void allocate(size_t a_new_length);

    /// Deallocate the buffer
    void free_buffer();

    /// Reset the buffer setting both write and read pointers to start
    void reset();

    /// Reset the buffer setting read pointers to start
    void reset() const;

    /// \param newPosition the new location of the m_read_pointer, or 0 to reset the read pointer to
    /// start of buffer.
    void set_read(const uint8_t* new_position = nullptr) const;

    /// \param bytes the number of bytes to increase the m_read_pointer by.
    void advance_read(size_t bytes) const;

    /// \param index the new index of the m_read_pointer, counted from the base pointer
    /// \return value of last read_index
    uint32_t read_index(uint32_t index) const;

    /// \return index of the m_read_pointer
    uint32_t read_index() const;

    /// \param newPosition the new location of the m_write_pointer, or 0 to reset the write pointer
    /// to start of buffer.
    void set_write(uint8_t* new_position = nullptr);

    /// \param bytes number of bytes to increase the current write pointer by.
    /// The memory that is skipped will be undefined.
    void advance_write(size_t bytes);

    /// \param index the new index of the m_write_pointer, counted from the base pointer
    /// \return value of last write_index
    uint32_t write_index(size_t index);

    /// \return index of the m_write_pointer
    uint32_t write_index() const;

    /// Ensures that at LEAST bytes after 'write_pointer' is available in the write buffer
    /// \param bytes the number of bytes that is needed after the current m_writer_pointer location
    /// \exception will cause an exception if the amount cannot be reserved for some reason
    void reserve_write(size_t bytes);

    /// Function that returns a pointer to the "end of the buffer", which can be used to avoid
    /// parsing functions writing beyond the allocated area.
    /// \return the pointer to the uint8_t succeeding the last legal uint8_t that is allocated.
    const uint8_t* end_pointer() const;

    /// Function that returns a pointer to the "start of buffer".
    /// \return the pointer to the first uint8_t of the buffer.
    const uint8_t* start_pointer() const;

    /// Function that returns a pointer to the "start of buffer".
    /// \return the pointer to the first uint8_t of the buffer.
    const uint8_t* data() const;

    /// Returns a pointer into the buffer at a specified index
    uint8_t* pointer(uint32_t index);

    /// Returns a const pointer into the buffer at a specified index
    const uint8_t* pointer(uint32_t index) const;

    /// This function returns the actual amount of allocated memory in the buffer.
    /// @return Length of the allocated data.
    uint32_t length() const;

    /// This function returns the total length of the entire readable buffer, from the start_pointer
    /// to the write_pointer. The function will return the same amount, regardless of how the
    /// read_pointer is manipulated.
    /// @return Length of the readable buffer
    uint32_t readable_length() const;

    /// This function returns the number of bytes that have been written, but not yet read.
    /// The function considers the manipulation of read_pointer.
    /// @return Number of bytes that are readable
    uint32_t unread_bytes() const;

    /// This function returns the number of bytes that are from the current write_point to the end
    /// of the buffer.
    /// @return Number of bytes that are writeable
    uint32_t writable_bytes() const;

    /// This function checks if the buffer has data at all in it at or not.
    /// @return true if the buffer is totally empty
    bool empty() const;

    /// This function describes if the buffer located inside is "owned" by this object, or is
    /// externally allocated. In the latter case, the buffer will not be reallocated or deleted when
    /// the buffer object goes out of scope.
    /// @return true if the buffer is locally owned, false otherwise
    bool owned() const;

    ic_cts::span<const uint8_t> span() const;

    ic_cts::span<const uint8_t> subspan(size_t pos, size_t count = dynamic_extent) const;

    /// This function is used to check if the buffer can be written to or not. A non-writeable
    /// buffer has been given a constant pointer as input data, which may point to a static
    /// structure that cannot be written.
    /// @return true if the buffer is writeable, false otherwise
    bool writeable() const;

    /// Safe write function
    void write(ic_cts::span<const uint8_t> data);

    /// Safe write function to copy from read position in a_data into the current write_position.
    void write(const Buffer& a_data);

    /// Safe write function
    void write(const uint8_t* data, size_t length);

    /// Safe read function
    void read(uint8_t* destination, size_t length) const;

    /// Read cursor that can be manipulated but not written to.
    /// A read_pointer can be increased even through a const object.
    mutable const uint8_t* m_read_pointer;

    /// Write cursor that can be manipulated to and written.
    /// No checking of out-of-bounds writing is done.
    /// This pointer also denotes the last legal byte that can be read.
    uint8_t* m_write_pointer;

    //! This function lets the user duplicate a buffer - this has a performance impact so it should
    //! be used with care.
    void duplicate(const Buffer& b);

    /// Get a pointer to the buffer data. If orphan is true, data
    /// ownership is transferred to the caller. Use with care.
    /// @return the data buffer, NULL if orphan is true and the Buffer does not own the data
    uint8_t* get_buffer(bool orphan = false);

    uint8_t at(uint32_t index) const;

    const_iterator begin() const;

    const_iterator end() const;

  private:
    uint8_t* m_pointer;  ///< pointer to the stored buffer

    uint32_t m_length;  ///< length of the stored buffer

    bool m_owned;  ///< flag indicating if the buffer is owned by this object and can be
                   ///< reallocated/freed safely

    bool m_writeable;  ///< flag indicating if the buffer can be written to
};

}  // namespace ic_cts

#include "detail/buffer.ic"  // IWYU pragma: export
