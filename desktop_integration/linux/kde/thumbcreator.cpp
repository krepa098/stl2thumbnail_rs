/*
Copyright (C) 2020  Paul Kremer

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "thumbcreator.h"
#include "stl2thumbnail.h"

#include <QImage>
#include <QString>
#include <QMimeDatabase>
#include <QFileInfo>

Q_LOGGING_CATEGORY(LOG_STL_THUMBS, "STLModelThumbs")

extern "C"
{
    // Factory method
    Q_DECL_EXPORT ThumbCreator *new_creator()
    {
        return new StlThumbCreator();
    }
};

StlThumbCreator::StlThumbCreator()
{
}

struct PicContainer
{
    s2t::PictureBuffer buffer;
};

void cleanup(void *data)
{
    auto container = static_cast<PicContainer *>(data);
    s2t::free_picture_buffer(&container->buffer);
    delete container;
}

bool StlThumbCreator::create(const QString &path, int width, int height, QImage &img)
{
    auto mime_type = QMimeDatabase().mimeTypeForFile(path);
    auto file_ext = QFileInfo(path).suffix().toLower();

    s2t::PictureBuffer pic;

    if (mime_type.inherits("model/stl") && file_ext == "stl")
    {
        s2t::RenderSettings settings;
        settings.width = width;
        settings.height = height;
        settings.timeout = 20000; // 20s
        settings.size_hint = false;
        settings.grid = false;
        settings.background_color[0] = 0.f; // r
        settings.background_color[1] = 0.f; // g
        settings.background_color[2] = 0.f; // b
        settings.background_color[3] = 0.f; // a

        // render
        pic = s2t::render_stl(path.toStdString().c_str(), settings);
    }
    else if (mime_type.inherits("text/x.gcode") || file_ext == "bgcode")
    {
        pic = s2t::extract_gcode_preview(path.toStdString().c_str(), width, height);
    }
    else if (mime_type.inherits("model/3mf"))
    {
        pic = s2t::extract_3mf_preview(path.toStdString().c_str(), width, height);
    }

    // failed?
    if (!pic.data)
        return false;

    // save the buffer in a container that stays around till it gets cleaned up
    // once 'img' goes out of scope
    auto container = new PicContainer{pic};

    // QImage owns the buffer and it has to stay valid throughout the life of the QImage
    img = QImage(pic.data, pic.width, pic.height, pic.stride, QImage::Format_RGBA8888, cleanup, container);

    return true;
}
