#ifndef __SERVICE_NET_H__
#define __SERVICE_NET_H__

#include "rust/cxx.h"
#include "packet.h"

struct ServiceWrapper;

namespace commlib
{
    class Service
    {
    public:
        int64_t id;
    };

    ////
    class ServiceNet : public Service
    {
    public:
        ServiceNet( int packetMemLimit )
        {
            
        }

        void OnConnection(struct ServiceWrapper* srv) {}

        void Init(struct ServiceWrapper* srv) {}

    public:
        int64_t num;
       
    };

} // namespace commlib

#endif // __SERVICE_NET_H__