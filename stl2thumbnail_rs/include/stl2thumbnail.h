#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace s2t {

struct PictureBuffer {
  /// data in rgba8888 format
  const uint8_t *data;
  /// length of the buffer
  uint32_t len;
  /// stride of the buffer
  uint32_t stride;
  /// depth of the buffer
  uint32_t depth;
};

struct RenderSettings {
  /// width of the image
  uint32_t width;
  /// height of the image
  uint32_t height;
  /// embed a size hint
  bool size_hint;
  /// max duration of the rendering, 0 to disable
  uint64_t timeout;
};

extern "C" {

/// Renders a mesh to a picture
/// Free the buffer with free_picture_buffer
PictureBuffer render(const char *path, RenderSettings settings);

/// Frees the memory of a PictureBuffer
void free_picture_buffer(PictureBuffer *buffer);

} // extern "C"

} // namespace s2t
